use crate::utils::{rnd::Rnd, time};

pub trait State<E>
where
    E: Env,
{
    /// `annealer`では最初のイテレーション以外はスコアの取得にこちらを使用される
    /// 差分計算などでスコアを効率的に計算できる場合は、この関数を再定義する
    fn get_score(&mut self, env: &E) -> f64 {
        self.calc_score(env)
    }

    /// スコアの完全な計算を行う
    fn calc_score(&mut self, env: &E) -> f64;
}

pub trait Env {}

pub trait NeighborHandler {
    type State: State<Self::Env>;
    type Env: Env;

    fn tag(&self) -> &'static str;
    fn apply(&mut self, state: &mut Self::State, env: &Self::Env, rnd: &mut Rnd) -> bool;
    fn revert(&mut self, state: &mut Self::State, env: &Self::Env, rnd: &mut Rnd);
}

pub trait NeighborType {
    type H: NeighborHandler;
    fn generate(&self) -> Self::H;
}

pub trait NeighborGenerator<N>
where
    N: NeighborType,
{
    fn generate(&self, progress: f64, rnd: &mut Rnd) -> N::H;
}

pub trait Criterion {
    fn adopt(
        &self,
        cur_score: f64,
        new_score: f64,
        cur_temp: f64,
        progress: f64,
        rnd: &mut Rnd,
    ) -> bool;
}

pub trait TemperatureScheduler {
    fn get_temp(&self, progress: f64) -> f64;
}

pub trait ProgressScheduler {
    fn start(&mut self) {}
    fn step(&mut self) {}
    fn get_progress(&self) -> f64;
}

pub struct IterationProgressScheduler {
    iteration: usize,
    cur_step: usize,
}

impl IterationProgressScheduler {
    pub fn new(iteration: usize) -> Self {
        IterationProgressScheduler {
            iteration,
            cur_step: 0,
        }
    }
}

impl ProgressScheduler for IterationProgressScheduler {
    fn step(&mut self) {
        self.cur_step += 1;
    }

    fn get_progress(&self) -> f64 {
        self.cur_step as f64 / self.iteration as f64
    }
}

pub struct SecondProgressScheduler {
    start_time: f64,
    seconds: f64,
}

impl SecondProgressScheduler {
    pub fn new(seconds: f64) -> Self {
        SecondProgressScheduler {
            start_time: 0.0,
            seconds,
        }
    }
}

impl ProgressScheduler for SecondProgressScheduler {
    fn start(&mut self) {
        self.start_time = time::elapsed_seconds();
    }

    fn get_progress(&self) -> f64 {
        (time::elapsed_seconds() - self.start_time) / self.seconds
    }
}

#[macro_export]
macro_rules! neighbor_impl {
    ($state_type:ident, $env_type:ident, $($variant:ident),+) => {
        enum NeighborHandlerImpl {
            $($variant($variant),)+
        }

        impl crate::annealer::types::NeighborHandler for NeighborHandlerImpl {
            type Env = $env_type;
            type State = $state_type;

            fn tag(&self) -> &'static str {
                match self {
                    $(Self::$variant(inner) => inner.tag(),)+
                }
            }

            fn apply(&mut self, state: &mut $state_type, env: &$env_type, rnd: &mut Rnd) -> bool {
                match self {
                    $(Self::$variant(inner) => inner.apply(state, env, rnd),)+
                }
            }

            fn revert(&mut self, state: &mut $state_type, env: &$env_type, rnd: &mut Rnd) {
                match self {
                    $(Self::$variant(inner) => inner.revert(state, env, rnd),)+
                }
            }
        }

        enum Neighbor {
            $($variant,)+
        }

        impl crate::annealer::types::NeighborType for Neighbor {
            type H = NeighborHandlerImpl;

            fn generate(&self) -> Self::H {
                match self {
                    $(Self::$variant => NeighborHandlerImpl::$variant($variant::generate()),)+
                }
            }
        }
    };
}
