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
