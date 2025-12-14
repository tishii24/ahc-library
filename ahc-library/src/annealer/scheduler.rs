use crate::{
    annealer::{
        prelude::{
            AnnealingCriterion, ExpTemperatureScheduler, IterationProgressScheduler,
            SecondProgressScheduler,
        },
        types::{Criterion, ProgressScheduler, TemperatureScheduler},
    },
    utils::random::Rnd,
};

enum AnnealerSchedulerStatus {
    NotStarted,
    InProgress,
    Finished,
}

/// Scheduler used for annealing process
///
/// Usage:
/// ```ignore
/// let mut scheduler = AnnealerScheduler::with_seconds(1e0, 1e-4, 1.0, true);
/// while scheduler.to_next_iter() {
///     let cur_score = state.get_score();
///
///     // do something
///
///     let new_score = state.get_score();
///
///     if scheduler.adopt(cur_score, new_score) {
///         // adopt
///     } else {
///         // revert
///     }
/// }
/// ```
pub struct AnnealerScheduler<C, T, P>
where
    C: Criterion,
    T: TemperatureScheduler,
    P: ProgressScheduler,
{
    status: AnnealerSchedulerStatus,
    criterion: C,
    temperature_scheduler: T,
    progress_scheduler: P,
    rnd: Rnd,
    iteration: usize,
    adopted: usize,
}

impl<C, T, P> AnnealerScheduler<C, T, P>
where
    C: Criterion,
    T: TemperatureScheduler,
    P: ProgressScheduler,
{
    pub fn new(criterion: C, temperature_scheduler: T, progress_scheduler: P) -> Self {
        AnnealerScheduler {
            status: AnnealerSchedulerStatus::NotStarted,
            criterion,
            temperature_scheduler,
            progress_scheduler,
            rnd: Rnd::new(24),
            iteration: 0,
            adopted: 0,
        }
    }

    pub fn adopt(&mut self, cur_score: f64, new_score: f64) -> bool {
        let progress = self.get_progress();
        let cur_temp = self.temperature_scheduler.get_temp(progress);
        let adopt = self
            .criterion
            .adopt(cur_score, new_score, cur_temp, progress, &mut self.rnd);
        if adopt {
            self.adopted += 1;
        }
        adopt
    }

    pub fn get_progress(&self) -> f64 {
        match self.status {
            AnnealerSchedulerStatus::NotStarted => panic!("Scheduler has not been started yet."),
            AnnealerSchedulerStatus::InProgress => self.progress_scheduler.get_progress(),
            AnnealerSchedulerStatus::Finished => 1.,
        }
    }

    pub fn to_next_iter(&mut self) -> bool {
        self.status = match self.status {
            AnnealerSchedulerStatus::NotStarted => {
                self.progress_scheduler.start();
                AnnealerSchedulerStatus::InProgress
            }
            AnnealerSchedulerStatus::InProgress => {
                self.progress_scheduler.step();
                if self.progress_scheduler.get_progress() >= 1. {
                    AnnealerSchedulerStatus::Finished
                } else {
                    AnnealerSchedulerStatus::InProgress
                }
            }
            AnnealerSchedulerStatus::Finished => AnnealerSchedulerStatus::Finished,
        };

        self.iteration += 1;
        matches!(self.status, AnnealerSchedulerStatus::InProgress)
    }

    #[inline]
    pub fn get_adopted(&self) -> usize {
        self.adopted
    }

    #[inline]
    pub fn get_iteration(&self) -> usize {
        self.iteration
    }
}

impl AnnealerScheduler<AnnealingCriterion, ExpTemperatureScheduler, SecondProgressScheduler> {
    pub fn with_seconds(
        start_temp: f64,
        end_temp: f64,
        seconds: f64,
        is_maximize: bool,
    ) -> AnnealerScheduler<AnnealingCriterion, ExpTemperatureScheduler, SecondProgressScheduler>
    {
        AnnealerScheduler::new(
            AnnealingCriterion::new(is_maximize),
            ExpTemperatureScheduler::new(start_temp, end_temp),
            SecondProgressScheduler::new(seconds),
        )
    }
}

impl AnnealerScheduler<AnnealingCriterion, ExpTemperatureScheduler, IterationProgressScheduler> {
    pub fn with_iterations(
        start_temp: f64,
        end_temp: f64,
        iteration: usize,
        is_maximize: bool,
    ) -> AnnealerScheduler<AnnealingCriterion, ExpTemperatureScheduler, IterationProgressScheduler>
    {
        AnnealerScheduler::new(
            AnnealingCriterion::new(is_maximize),
            ExpTemperatureScheduler::new(start_temp, end_temp),
            IterationProgressScheduler::new(iteration),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::annealer::prelude::AnnealerScheduler;

    #[test]
    fn test_annealer_scheduler_with_iterations() {
        const ITERATIONS: usize = 100;
        let mut scheduler = AnnealerScheduler::with_iterations(1e0, 1e-4, ITERATIONS, true);
        let mut iterations = 0;
        while scheduler.to_next_iter() {
            iterations += 1;
        }

        assert_eq!(iterations, ITERATIONS);
    }

    #[test]
    fn test_annealer_scheduler_with_seconds() {
        const SECONDS: f64 = 0.3;
        let mut scheduler = AnnealerScheduler::with_seconds(1e0, 1e-4, SECONDS, true);
        let start = std::time::Instant::now();
        let mut iterations = 0;
        while scheduler.to_next_iter() {
            iterations += 1;
        }
        let elapsed = start.elapsed().as_secs_f64();
        assert!((elapsed - SECONDS).abs() < 0.1);
        assert!(iterations > 0);
    }
}
