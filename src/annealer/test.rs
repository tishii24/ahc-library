#[cfg(test)]
mod tests {
    use crate::annealer::annealer::*;
    use crate::annealer::components::{Mutator, WeightedNeighborGenerator};
    use crate::annealer::types::*;
    use crate::neighbor_impl;

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

        fn apply(&self, state: &mut StateImpl, _: &EnvImpl) {
            state.c += self.a;
        }

        fn revert(&self, state: &mut StateImpl, _: &EnvImpl) {
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

        fn apply(&self, state: &mut StateImpl, _: &EnvImpl) {
            state.c += self.b;
        }

        fn revert(&self, state: &mut StateImpl, _: &EnvImpl) {
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

        let (state, env) = (annealer.state, annealer.env);
        assert_eq!(state.calc_score(&env), 0.);
    }
}
