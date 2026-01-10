pub mod neighbor_generator {
    use crate::{
        annealer::types::{NeighborGenerator, NeighborType},
        utils::random::Random,
    };

    pub struct WeightedNeighborGenerator<N>
    where
        N: NeighborType,
    {
        neighbors: Vec<(N, f64)>,
    }

    impl<N> WeightedNeighborGenerator<N>
    where
        N: NeighborType,
    {
        pub fn new(mut neighbors: Vec<(N, f64)>) -> WeightedNeighborGenerator<N> {
            let total_weight: f64 = neighbors.iter().map(|(_, weight)| *weight).sum();
            let mut cum_weight = 0.;
            for (_, weight) in neighbors.iter_mut() {
                assert!(*weight >= 0.0);
                cum_weight += *weight / total_weight;
                *weight = cum_weight;
            }

            WeightedNeighborGenerator { neighbors }
        }
    }

    impl<N> NeighborGenerator<N> for WeightedNeighborGenerator<N>
    where
        N: NeighborType,
    {
        fn generate(&self, _: f64, rnd: &mut impl Random) -> N::H {
            let p = rnd.nextf();
            for (n, cum_weight) in self.neighbors.iter() {
                if p < *cum_weight {
                    return n.setup();
                }
            }
            unreachable!()
        }
    }
}

pub mod criterion {
    use crate::{annealer::types::Criterion, utils::random::Random};

    pub struct HillClimbingCriterion {
        is_maximize: bool,
    }

    impl HillClimbingCriterion {
        pub fn new(is_maximize: bool) -> Self {
            HillClimbingCriterion { is_maximize }
        }
    }

    impl Criterion for HillClimbingCriterion {
        fn adopt(
            &self,
            cur_score: f64,
            new_score: f64,
            _: f64,
            _: f64,
            _: &mut impl Random,
        ) -> bool {
            if self.is_maximize {
                new_score >= cur_score
            } else {
                new_score <= cur_score
            }
        }
    }

    pub struct AnnealingCriterion {
        is_maximize: bool,
    }

    impl AnnealingCriterion {
        pub fn new(is_maximize: bool) -> Self {
            AnnealingCriterion { is_maximize }
        }
    }

    impl Criterion for AnnealingCriterion {
        fn adopt(
            &self,
            cur_score: f64,
            new_score: f64,
            cur_temp: f64,
            _: f64,
            rnd: &mut impl Random,
        ) -> bool {
            let sign = self.is_maximize as i32 * 2 - 1;
            let score_diff = sign as f64 * (new_score - cur_score);
            if score_diff > 0. {
                return true;
            }
            let prob = (score_diff / cur_temp).exp();
            rnd.nextf() < prob
        }
    }
}

pub mod temperature_scheduler {
    use crate::annealer::types::TemperatureScheduler;

    pub struct ExpTemperatureScheduler {
        start_temp: f64,
        end_temp: f64,
    }

    impl ExpTemperatureScheduler {
        pub fn new(start_temp: f64, end_temp: f64) -> Self {
            ExpTemperatureScheduler {
                start_temp,
                end_temp,
            }
        }
    }

    impl TemperatureScheduler for ExpTemperatureScheduler {
        fn get_temp(&self, progress: f64) -> f64 {
            self.start_temp.powf(1. - progress) * self.end_temp.powf(progress)
        }
    }
}

pub mod progress_scheduler {
    use crate::{annealer::types::ProgressScheduler, utils::time};

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
}

pub mod callback {
    use crate::annealer::types::{Callback, Env, State};
    use std::marker::PhantomData;

    pub struct RestoreBestStateCallback<S, E>
    where
        S: State<E>,
        E: Env,
    {
        is_maximize: bool,
        patience: usize,
        last_update_step: usize,
        best_state: Option<S>,
        _phantom: PhantomData<E>,
    }

    impl<S, E> RestoreBestStateCallback<S, E>
    where
        S: State<E>,
        E: Env,
    {
        pub fn new(patience: usize, is_maximize: bool) -> Self {
            RestoreBestStateCallback {
                last_update_step: 0,
                patience,
                best_state: None,
                is_maximize,
                _phantom: PhantomData,
            }
        }
    }

    impl<S, E> Callback<S, E> for RestoreBestStateCallback<S, E>
    where
        S: State<E>,
        E: Env,
    {
        fn on_after_step(&mut self, step: usize, progress: f64, state: &mut S, env: &E) {
            let should_update = match &mut self.best_state {
                None => true,
                Some(s) => {
                    let cur_score = s.get_score(env, progress);
                    let new_score = state.get_score(env, progress);
                    self.is_maximize == (new_score > cur_score)
                }
            };
            if should_update {
                self.best_state = Some(state.clone());
                self.last_update_step = step;
                return;
            }

            if step < self.last_update_step + self.patience {
                return;
            }

            *state = self.best_state.clone().unwrap();
            self.last_update_step = step;
        }
    }

    pub struct ReturnBestStateCallback<S, E>
    where
        S: State<E>,
        E: Env,
    {
        is_maximize: bool,
        best_state: Option<S>,
        _phantom: PhantomData<E>,
    }

    impl<S, E> ReturnBestStateCallback<S, E>
    where
        S: State<E>,
        E: Env,
    {
        pub fn new(is_maximize: bool) -> Self {
            ReturnBestStateCallback {
                best_state: None,
                is_maximize,
                _phantom: PhantomData,
            }
        }
    }

    impl<S, E> Callback<S, E> for ReturnBestStateCallback<S, E>
    where
        S: State<E>,
        E: Env,
    {
        fn on_after_step(&mut self, _: usize, progress: f64, state: &mut S, env: &E) {
            let should_update = match &mut self.best_state {
                None => true,
                Some(s) => {
                    let cur_score = s.get_score(env, progress);
                    let new_score = state.get_score(env, progress);
                    self.is_maximize == (new_score > cur_score)
                }
            };
            if should_update {
                self.best_state = Some(state.clone());
            }
        }

        fn on_finish(&mut self, state: &mut S, _env: &E) {
            if let Some(best_state) = self.best_state.as_mut() {
                *state = best_state.clone();
            }
        }
    }
}
