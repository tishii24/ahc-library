#[cfg(test)]
mod test_single_variable {
    use crate::annealer::annealer::*;
    use crate::annealer::components::{
        criterion::HillClimbingCriterion, neighbor_generator::WeightedNeighborGenerator,
        progress_scheduler::IterationProgressScheduler,
        temperature_scheduler::ExpTemperatureScheduler,
    };
    use crate::annealer::types::*;
    use crate::neighbor_impl;
    use crate::utils::rnd::Rnd;

    #[derive(Clone)]
    struct StateImpl {
        c: f64,
    }

    impl State<EnvImpl> for StateImpl {
        fn get_score(&mut self, env: &EnvImpl, _: f64) -> f64 {
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
        fn setup() -> NeighborA {
            NeighborA { a: 1. }
        }

        fn apply(&mut self, state: &mut StateImpl, _: &EnvImpl, _: &mut Rnd) -> bool {
            state.c += self.a;
            true
        }

        fn revert(&mut self, state: &mut StateImpl, _: &EnvImpl, _: &mut Rnd) {
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
        fn setup() -> NeighborB {
            NeighborB { b: 1. }
        }

        fn apply(&mut self, state: &mut StateImpl, _: &EnvImpl, _: &mut Rnd) -> bool {
            state.c += self.b;
            true
        }

        fn revert(&mut self, state: &mut StateImpl, _: &EnvImpl, _: &mut Rnd) {
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
            (Neighbor::NeighborA, 0.2),
            (Neighbor::NeighborB, 2.0),
        ]);
        let mutator = Mutator::new(generator);
        let mut annealer = Annealer::new(
            state,
            env,
            mutator,
            IterationProgressScheduler::new(1_000),
            HillClimbingCriterion::new(false),
            ExpTemperatureScheduler::new(1e0, 1e-4),
            vec![],
            AnnealerConfig {
                mode: AnnealerMode::Release,
            },
        );
        annealer.run();

        let (mut state, env, statistics) = (annealer.state, annealer.env, annealer.logger);
        assert_eq!(state.get_score(&env, 0.), 0.);
        statistics.print();
    }
}

#[cfg(test)]
mod test_knapsack {
    use crate::annealer::annealer::*;
    use crate::annealer::components::{
        criterion::AnnealingCriterion, neighbor_generator::WeightedNeighborGenerator,
        progress_scheduler::SecondProgressScheduler,
        temperature_scheduler::ExpTemperatureScheduler,
    };
    use crate::annealer::types::*;
    use crate::neighbor_impl;
    use crate::utils::rnd::Rnd;

    #[derive(Clone)]
    struct StateImpl {
        value_sum: f64,
        weight_sum: f64,
        used: Vec<bool>,
    }

    impl State<EnvImpl> for StateImpl {
        fn get_score(&mut self, env: &EnvImpl, _: f64) -> f64 {
            if self.weight_sum > env.weight_limit {
                return 0.;
            }
            self.value_sum
        }
    }

    struct EnvImpl {
        items: Vec<(f64, f64)>,
        weight_limit: f64,
    }

    impl Env for EnvImpl {}

    struct ToggleOne {
        i: Option<usize>,
    }

    impl ToggleOne {
        fn setup() -> ToggleOne {
            ToggleOne { i: None }
        }

        fn apply(&mut self, state: &mut StateImpl, env: &EnvImpl, rnd: &mut Rnd) -> bool {
            self.i = Some(rnd.gen_index(env.items.len()));
            let i = self.i.unwrap();
            if state.used[i] {
                state.weight_sum -= env.items[i].0;
                state.value_sum -= env.items[i].1;
            } else {
                state.weight_sum += env.items[i].0;
                state.value_sum += env.items[i].1;
            }
            state.used[i] = !state.used[i];
            true
        }

        fn revert(&mut self, state: &mut StateImpl, env: &EnvImpl, _: &mut Rnd) {
            let i = self.i.unwrap();
            if state.used[i] {
                state.weight_sum -= env.items[i].0;
                state.value_sum -= env.items[i].1;
            } else {
                state.weight_sum += env.items[i].0;
                state.value_sum += env.items[i].1;
            }
            state.used[i] = !state.used[i];
        }

        fn tag(&self) -> &'static str {
            "ToggleOne"
        }
    }

    neighbor_impl!(StateImpl, EnvImpl, ToggleOne);

    #[test]
    fn test_run() {
        let env = EnvImpl {
            items: vec![(1., 1.), (2., 2.), (3., 3.), (4., 4.), (5., 5.)],
            weight_limit: 10.,
        };
        let state = StateImpl {
            value_sum: 0.,
            weight_sum: 0.,
            used: vec![false; env.items.len()],
        };
        let generator = WeightedNeighborGenerator::new(vec![(Neighbor::ToggleOne, 0.8)]);
        let mutator = Mutator::new(generator);
        let mut annealer = Annealer::new(
            state,
            env,
            mutator,
            SecondProgressScheduler::new(0.1),
            AnnealingCriterion::new(true),
            ExpTemperatureScheduler::new(1e0, 1e-4),
            vec![],
            AnnealerConfig {
                mode: AnnealerMode::Release,
            },
        );
        annealer.run();

        let (mut state, env, statistics) = (annealer.state, annealer.env, annealer.logger);
        assert_eq!(state.get_score(&env, 1.), 10.);
        statistics.print();
    }
}
