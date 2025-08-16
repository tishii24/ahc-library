#![allow(unused_variables, unused_macros)]

/*
TODO:
- デバッグモード
- 温度自動調整
- 統計出力
- 近傍確率の時間に応じた調整
- 近傍ごとの受諾確率調整
- ベスト解出力
- 温度スケジュール
- 重みの自動計算
*/

pub trait State<E>
where
    E: Env,
{
    fn calc_score(&self, env: &E) -> f64;
}

pub trait Env {}

struct Annealer<N, G>
where
    G: NeighborGenerator<N>,
    N: Neighbors,
{
    state: <N::H as NeighborHandler>::State,
    env: <N::H as NeighborHandler>::Env,
    mutator: Mutator<G, N>,
    config: AnnealerConfig,
}

impl<N, G> Annealer<N, G>
where
    G: NeighborGenerator<N>,
    N: Neighbors,
{
    fn new(
        state: <N::H as NeighborHandler>::State,
        env: <N::H as NeighborHandler>::Env,
        mutator: Mutator<G, N>,
        config: AnnealerConfig,
    ) -> Annealer<N, G> {
        Annealer {
            state,
            env,
            mutator,
            config,
        }
    }

    fn run(&mut self) {
        let mut cur_score = self.state.calc_score(&self.env);
        for t in 0..self.config.iteration {
            self.mutator.commit(&mut self.state, &self.env);

            let new_score = self.state.calc_score(&self.env);
            if self.adopt(cur_score, new_score) {
                cur_score = new_score;
            } else {
                self.mutator.revert(&mut self.state, &self.env);
            }

            if t % self.config.log_interval == 0 {
                eprintln!("[{:5}] {} -> {}", t, cur_score, new_score);
            }
        }
    }

    fn adopt(&self, cur_score: f64, new_score: f64) -> bool {
        new_score < cur_score
    }

    fn register_result(&self, result: bool, improved: bool, score_delta: f32) {
        todo!()
    }
}

struct AnnealerConfig {
    start_temp: f64,
    end_temp: f64,
    iteration: usize,
    is_maximize: bool,
    log_interval: usize,
}

trait NeighborHandler {
    type State: State<Self::Env>;
    type Env: Env;

    fn tag(&self) -> &'static str;
    fn commit(&self, state: &mut Self::State, env: &Self::Env);
    fn revert(&self, state: &mut Self::State, env: &Self::Env);
}

trait Neighbors {
    type H: NeighborHandler;
    fn generate(&self) -> Self::H;
}

trait NeighborGenerator<N>
where
    N: Neighbors,
{
    fn generate(&self) -> N::H;
}

struct WeightedNeighborGenerator<N>
where
    N: Neighbors,
{
    neighbors: Vec<(N, f32)>,
}

impl<N> WeightedNeighborGenerator<N>
where
    N: Neighbors,
{
    fn new(neighbors: Vec<(N, f32)>) -> WeightedNeighborGenerator<N> {
        WeightedNeighborGenerator { neighbors }
    }
}

impl<N> NeighborGenerator<N> for WeightedNeighborGenerator<N>
where
    N: Neighbors,
{
    fn generate(&self) -> N::H {
        // TODO: improve
        self.neighbors[0].0.generate()
    }
}

struct Mutator<G, N>
where
    G: NeighborGenerator<N>,
    N: Neighbors,
{
    generator: G,
    last_neighbor: Option<N::H>,
}

impl<G, N> Mutator<G, N>
where
    G: NeighborGenerator<N>,
    N: Neighbors,
{
    fn new(generator: G) -> Mutator<G, N> {
        Mutator {
            generator,
            last_neighbor: None,
        }
    }

    fn commit(
        &mut self,
        state: &mut <N::H as NeighborHandler>::State,
        env: &<N::H as NeighborHandler>::Env,
    ) {
        let n = self.generator.generate();
        n.commit(state, env);
        self.last_neighbor = Some(n);
    }

    fn revert(
        &mut self,
        state: &mut <N::H as NeighborHandler>::State,
        env: &<N::H as NeighborHandler>::Env,
    ) {
        let last_neighbor = self
            .last_neighbor
            .take()
            .expect("expect last neighbor being set before revert");
        last_neighbor.revert(state, env);
    }

    fn get_last_tag(&self) -> Option<&'static str> {
        if let Some(last_neighbor) = &self.last_neighbor {
            Some(last_neighbor.tag())
        } else {
            None
        }
    }
}

macro_rules! neighbor_impl {
    ($state_type:ident, $env_type:ident, $($variant:ident),+) => {
        enum NeighborHandlerImpl {
            $($variant($variant),)+
        }

        impl NeighborHandler for NeighborHandlerImpl {
            type Env = $env_type;
            type State = $state_type;

            fn tag(&self) -> &'static str {
                match self {
                    $(Self::$variant(inner) => inner.tag(),)+
                }
            }

            fn commit(&self, state: &mut $state_type, env: &$env_type) {
                match self {
                    $(Self::$variant(inner) => inner.commit(state, env),)+
                }
            }

            fn revert(&self, state: &mut $state_type, env: &$env_type) {
                match self {
                    $(Self::$variant(inner) => inner.revert(state, env),)+
                }
            }
        }

        enum Neighbor {
            $($variant,)+
        }

        impl Neighbors for Neighbor {
            type H = NeighborHandlerImpl;

            fn generate(&self) -> Self::H {
                match self {
                    $(Self::$variant => NeighborHandlerImpl::$variant($variant::generate()),)+
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::annealer::*;

    struct StateImpl {
        c: f64,
    }

    impl State<EnvImpl> for StateImpl {
        fn calc_score(&self, env: &EnvImpl) -> f64 {
            (self.c - env.d).powf(2.)
        }
    }

    struct EnvImpl {
        d: f64,
    }

    impl Env for EnvImpl {}

    struct NeighborA {
        a: f64,
    }

    impl NeighborA {
        fn generate() -> NeighborA {
            NeighborA { a: 1. }
        }

        fn commit(&self, state: &mut StateImpl, env: &EnvImpl) {
            state.c += self.a;
        }

        fn revert(&self, state: &mut StateImpl, env: &EnvImpl) {
            state.c -= self.a;
        }

        fn tag(&self) -> &'static str {
            "NeighborA"
        }
    }

    struct NeighborB {
        b: f64,
    }

    impl NeighborB {
        fn generate() -> NeighborB {
            NeighborB { b: 1. }
        }

        fn commit(&self, state: &mut StateImpl, env: &EnvImpl) {
            state.c += self.b;
        }

        fn revert(&self, state: &mut StateImpl, env: &EnvImpl) {
            state.c -= self.b;
        }

        fn tag(&self) -> &'static str {
            "NeighborB"
        }
    }

    neighbor_impl!(StateImpl, EnvImpl, NeighborA, NeighborB);

    #[test]
    fn test_run() {
        let state = StateImpl { c: -10. };
        let env = EnvImpl { d: 0. };
        let generator = WeightedNeighborGenerator::new(vec![
            (Neighbor::NeighborA, 0.8),
            (Neighbor::NeighborB, 0.8),
        ]);
        let mutator = Mutator::new(generator);
        let config = AnnealerConfig {
            start_temp: 1e1,
            end_temp: 1e-3,
            is_maximize: false,
            iteration: 1000,
            log_interval: 100,
        };
        let mut annealer = Annealer::new(state, env, mutator, config);
        annealer.run();
    }
}
