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

pub struct AnnealerSchedulerWithStatistics<C, T, P>
where
    C: Criterion,
    T: TemperatureScheduler,
    P: ProgressScheduler,
{
    inner: AnnealerScheduler<C, T, P>,
    iteration: Vec<usize>,
    adopted: Vec<usize>,
}

impl<C, T, P> AnnealerSchedulerWithStatistics<C, T, P>
where
    C: Criterion,
    T: TemperatureScheduler,
    P: ProgressScheduler,
{
    pub fn new(inner: AnnealerScheduler<C, T, P>, n_types: usize) -> Self {
        assert!(n_types > 0, "n_types must be > 0");
        Self {
            inner,
            iteration: vec![0; n_types],
            adopted: vec![0; n_types],
        }
    }

    pub fn to_next_iter(&mut self) -> bool {
        self.inner.to_next_iter()
    }

    pub fn adopt(&mut self, t: usize, cur_score: f64, new_score: f64) -> bool {
        self.iteration[t] += 1;
        let adopted = self.inner.adopt(cur_score, new_score);
        if adopted {
            self.adopted[t] += 1;
        }
        adopted
    }

    #[inline]
    pub fn get_adopted(&self) -> &[usize] {
        &self.adopted
    }

    #[inline]
    pub fn get_iteration(&self) -> &[usize] {
        &self.iteration
    }

    #[inline]
    pub fn get_progress(&self) -> f64 {
        self.inner.get_progress()
    }

    /// Also prints totals and current progress.
    pub fn print_statistics(&self) {
        let n = self.iteration.len();
        let total_iter: usize = self.iteration.iter().copied().sum();
        let total_adopt: usize = self.adopted.iter().copied().sum();
        let total_rate = if total_iter == 0 {
            0.0
        } else {
            (total_adopt as f64) / (total_iter as f64) * 100.0
        };

        eprintln!("=== Annealer statistics ===");
        eprintln!("progress: {:.4}", self.get_progress());
        eprintln!("types: {}", n);
        eprintln!(
            "{:>6} | {:>12} | {:>12} | {:>10}",
            "type", "adopted", "iteration", "rate %"
        );
        eprintln!("{}", "-".repeat(6 + 3 + 12 + 3 + 12 + 3 + 10));
        for t in 0..n {
            let it = self.iteration[t];
            let ad = self.adopted[t];
            let rate = if it == 0 {
                0.0
            } else {
                (ad as f64) / (it as f64) * 100.0
            };
            eprintln!("{:>6} | {:>12} | {:>12} | {:>10.2}", t, ad, it, rate);
        }
        eprintln!("{}", "-".repeat(6 + 3 + 12 + 3 + 12 + 3 + 10));
        eprintln!(
            "{:>6} | {:>12} | {:>12} | {:>10.2}",
            "total", total_adopt, total_iter, total_rate
        );
    }
}

impl
    AnnealerSchedulerWithStatistics<
        AnnealingCriterion,
        ExpTemperatureScheduler,
        SecondProgressScheduler,
    >
{
    pub fn with_seconds(
        start_temp: f64,
        end_temp: f64,
        seconds: f64,
        is_maximize: bool,
        n_types: usize,
    ) -> Self {
        let inner = AnnealerScheduler::with_seconds(start_temp, end_temp, seconds, is_maximize);
        Self::new(inner, n_types)
    }
}

impl
    AnnealerSchedulerWithStatistics<
        AnnealingCriterion,
        ExpTemperatureScheduler,
        IterationProgressScheduler,
    >
{
    pub fn with_iterations(
        start_temp: f64,
        end_temp: f64,
        iteration: usize,
        is_maximize: bool,
        n_types: usize,
    ) -> Self {
        let inner =
            AnnealerScheduler::with_iterations(start_temp, end_temp, iteration, is_maximize);
        Self::new(inner, n_types)
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

    #[test]
    fn test_annealer_scheduler_with_statistics() {
        const ITERATIONS: usize = 50;
        const N_TYPES: usize = 3;
        let mut scheduler =
            crate::annealer::scheduler::AnnealerSchedulerWithStatistics::with_iterations(
                1e0, 1e-4, ITERATIONS, true, N_TYPES,
            );
        let mut total_iterations = 0;
        while scheduler.to_next_iter() {
            scheduler.adopt(total_iterations % N_TYPES, 1., 2.);
            total_iterations += 1;
        }

        let recorded_iterations: usize = scheduler.get_iteration().iter().sum();
        scheduler.print_statistics();
        assert_eq!(total_iterations, recorded_iterations);
    }
}
