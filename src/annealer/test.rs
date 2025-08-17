#[cfg(test)]
mod test_single_variable {
    use crate::annealer::annealer::*;
    use crate::annealer::components::{
        ExpScheduler, HillClimbingCriterion, Mutator, WeightedNeighborGenerator,
    };
    use crate::annealer::types::*;
    use crate::neighbor_impl;
    use crate::utils::rnd::Rnd;

    struct StateImpl {
        c: f64,
    }

    impl State<EnvImpl> for StateImpl {
        fn calc_score(&mut self, env: &EnvImpl) -> f64 {
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
        fn generate() -> NeighborB {
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
            (Neighbor::NeighborB, 0.8),
        ]);
        let mutator = Mutator::new(generator);
        let config = AnnealerConfig {
            iteration: 1000,
            log_interval: 100,
        };
        let scheduler = ExpScheduler::new(1e0, 1e-4);
        let criterion = HillClimbingCriterion::new(false);
        let rng = Rnd::new(42);
        let mut annealer = Annealer::new(state, env, mutator, criterion, scheduler, config, rng);
        annealer.run();

        let (mut state, env, statistics) = (annealer.state, annealer.env, annealer.log_store);
        assert_eq!(state.calc_score(&env), 0.);
        statistics.print();
    }
}

#[cfg(test)]
mod test_knapsack {
    use crate::annealer::annealer::*;
    use crate::annealer::components::{
        AnnealingCriterion, ExpScheduler, Mutator, WeightedNeighborGenerator,
    };
    use crate::annealer::types::*;
    use crate::neighbor_impl;
    use crate::utils::rnd::Rnd;

    struct StateImpl {
        value_sum: f64,
        weight_sum: f64,
        used: Vec<bool>,
    }

    impl State<EnvImpl> for StateImpl {
        fn get_score(&mut self, env: &EnvImpl) -> f64 {
            if self.weight_sum > env.weight_limit {
                return 0.;
            }
            self.value_sum
        }

        fn calc_score(&mut self, env: &EnvImpl) -> f64 {
            self.value_sum = 0.;
            self.weight_sum = 0.;

            for (used, &(weight, value)) in self.used.iter().zip(&env.items) {
                if *used {
                    self.value_sum += value;
                    self.weight_sum += weight;
                }
            }

            self.get_score(env)
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
        fn generate() -> ToggleOne {
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
        let config = AnnealerConfig {
            iteration: 1000,
            log_interval: 100,
        };
        let scheduler = ExpScheduler::new(1e0, 1e-4);
        let criterion = AnnealingCriterion::new(true);
        let rng = Rnd::new(42);
        let mut annealer = Annealer::new(state, env, mutator, criterion, scheduler, config, rng);
        annealer.run();

        let (mut state, env, statistics) = (annealer.state, annealer.env, annealer.log_store);
        assert_eq!(state.calc_score(&env), 10.);
        statistics.print();
    }
}
