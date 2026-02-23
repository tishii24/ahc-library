use crate::{
    annealer::{
        components::criterion::AnnealingCriterion,
        components::progress_scheduler::{IterationProgressScheduler, SecondProgressScheduler},
        components::temperature_scheduler::ExpTemperatureScheduler,
        types::{Criterion, ProgressScheduler, TemperatureScheduler},
    },
    utils::random::Random,
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
pub struct AnnealerScheduler<C, T, P, R>
where
    C: Criterion,
    T: TemperatureScheduler,
    P: ProgressScheduler,
    R: Random,
{
    status: AnnealerSchedulerStatus,
    criterion: C,
    temperature_scheduler: T,
    progress_scheduler: P,
    rnd: R,
}

impl<C, T, P, R> AnnealerScheduler<C, T, P, R>
where
    C: Criterion,
    T: TemperatureScheduler,
    P: ProgressScheduler,
    R: Random,
{
    pub fn new(criterion: C, temperature_scheduler: T, progress_scheduler: P, rnd: R) -> Self {
        AnnealerScheduler {
            status: AnnealerSchedulerStatus::NotStarted,
            criterion,
            temperature_scheduler,
            progress_scheduler,
            rnd,
        }
    }

    pub fn adopt(&mut self, cur_score: f64, new_score: f64) -> bool {
        let progress = self.get_progress();
        let cur_temp = self.temperature_scheduler.get_temp(progress);
        self.criterion
            .adopt(cur_score, new_score, cur_temp, progress, &mut self.rnd)
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

        matches!(self.status, AnnealerSchedulerStatus::InProgress)
    }
}

impl<R: Random>
    AnnealerScheduler<AnnealingCriterion, ExpTemperatureScheduler, SecondProgressScheduler, R>
{
    pub fn with_seconds(
        start_temp: f64,
        end_temp: f64,
        seconds: f64,
        is_maximize: bool,
        rnd: R,
    ) -> AnnealerScheduler<AnnealingCriterion, ExpTemperatureScheduler, SecondProgressScheduler, R>
    {
        AnnealerScheduler::new(
            AnnealingCriterion::new(is_maximize),
            ExpTemperatureScheduler::new(start_temp, end_temp),
            SecondProgressScheduler::new(seconds),
            rnd,
        )
    }
}

impl<R: Random>
    AnnealerScheduler<AnnealingCriterion, ExpTemperatureScheduler, IterationProgressScheduler, R>
{
    pub fn with_iterations(
        start_temp: f64,
        end_temp: f64,
        iteration: usize,
        is_maximize: bool,
        rnd: R,
    ) -> AnnealerScheduler<AnnealingCriterion, ExpTemperatureScheduler, IterationProgressScheduler, R>
    {
        AnnealerScheduler::new(
            AnnealingCriterion::new(is_maximize),
            ExpTemperatureScheduler::new(start_temp, end_temp),
            IterationProgressScheduler::new(iteration),
            rnd,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::random::XorShift32;

    use super::*;

    #[test]
    fn test_annealer_scheduler_with_iterations() {
        const ITERATIONS: usize = 100;
        let mut scheduler =
            AnnealerScheduler::with_iterations(1e0, 1e-4, ITERATIONS, true, XorShift32::new(24));
        let mut iterations = 0;
        while scheduler.to_next_iter() {
            iterations += 1;
        }

        assert_eq!(iterations, ITERATIONS);
    }

    #[test]
    fn test_annealer_scheduler_with_seconds() {
        const SECONDS: f64 = 0.3;
        let mut scheduler =
            AnnealerScheduler::with_seconds(1e0, 1e-4, SECONDS, true, XorShift32::new(24));
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
