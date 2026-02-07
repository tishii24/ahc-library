use crate::{
    annealer::{
        prelude::{
            AnnealingCriterion, ExpTemperatureScheduler, IterationProgressScheduler,
            SecondProgressScheduler,
        },
        types::{Criterion, ProgressScheduler, TemperatureScheduler},
    },
    utils::random::Random,
};

const NUM_PHASE: usize = 3;

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

pub struct AnnealerSchedulerWithStatistics<C, T, P, R>
where
    C: Criterion,
    T: TemperatureScheduler,
    P: ProgressScheduler,
    R: Random,
{
    inner: AnnealerScheduler<C, T, P, R>,
    iteration: Vec<[usize; NUM_PHASE]>,
    adopted: Vec<[usize; NUM_PHASE]>,
}

impl<C, T, P, R> AnnealerSchedulerWithStatistics<C, T, P, R>
where
    C: Criterion,
    T: TemperatureScheduler,
    P: ProgressScheduler,
    R: Random,
{
    pub fn new(inner: AnnealerScheduler<C, T, P, R>, n_types: usize) -> Self {
        assert!(n_types > 0, "n_types must be > 0");
        Self {
            inner,
            iteration: vec![[0; NUM_PHASE]; n_types],
            adopted: vec![[0; NUM_PHASE]; n_types],
        }
    }

    pub fn to_next_iter(&mut self) -> bool {
        self.inner.to_next_iter()
    }

    pub fn adopt(&mut self, t: usize, cur_score: f64, new_score: f64) -> bool {
        // phase index: 0..=NUM_PHASE-1 based on current progress
        let progress = self.inner.get_progress();
        let phase = ((progress * NUM_PHASE as f64).floor() as usize).min(NUM_PHASE - 1);

        self.iteration[t][phase] += 1;
        let adopted = self.inner.adopt(cur_score, new_score);
        if adopted {
            self.adopted[t][phase] += 1;
        }
        adopted
    }

    #[inline]
    pub fn get_adopted(&self) -> &[[usize; NUM_PHASE]] {
        &self.adopted
    }

    #[inline]
    pub fn get_iteration(&self) -> &[[usize; NUM_PHASE]] {
        &self.iteration
    }

    #[inline]
    pub fn get_progress(&self) -> f64 {
        self.inner.get_progress()
    }

    pub fn eprint_statistics(&self) {
        let n = self.iteration.len();
        let total_iter: usize = self
            .iteration
            .iter()
            .map(|row| row.iter().sum::<usize>())
            .sum();
        let total_adopt: usize = self
            .adopted
            .iter()
            .map(|row| row.iter().sum::<usize>())
            .sum();
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
            let it: usize = self.iteration[t].iter().sum();
            let ad: usize = self.adopted[t].iter().sum();
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

        // Per-type x per-phase acceptance rate (%)
        eprintln!("");
        eprintln!("per-type per-phase acceptance rate (%)");
        // header
        {
            use std::fmt::Write as _;
            let mut header = String::new();
            let _ = write!(&mut header, "{:>6} |", "type");
            for p in 0..NUM_PHASE {
                let _ = write!(&mut header, " {:>8}", format!("p{}", p));
            }
            let _ = write!(&mut header, " | {:>8}", "avg");
            eprintln!("{}", header);
        }
        let sep_len = 6 + 3 + NUM_PHASE * 9 + 3 + 8;
        eprintln!("{}", "-".repeat(sep_len));
        for t in 0..n {
            let mut avg_it = 0usize;
            let mut avg_ad = 0usize;
            let mut rates = vec![0.0f64; NUM_PHASE];
            for p in 0..NUM_PHASE {
                let it = self.iteration[t][p];
                let ad = self.adopted[t][p];
                avg_it += it;
                avg_ad += ad;
                rates[p] = if it == 0 {
                    0.0
                } else {
                    (ad as f64) / (it as f64) * 100.0
                };
            }
            let avg_rate = if avg_it == 0 {
                0.0
            } else {
                (avg_ad as f64) / (avg_it as f64) * 100.0
            };
            use std::fmt::Write as _;
            let mut line = String::new();
            let _ = write!(&mut line, "{:>6} |", t);
            for p in 0..NUM_PHASE {
                let _ = write!(&mut line, " {:>8.2}", rates[p]);
            }
            let _ = write!(&mut line, " | {:>8.2}", avg_rate);
            eprintln!("{}", line);
        }

        // Per-phase totals across all types
        let mut phase_iter = [0usize; NUM_PHASE];
        let mut phase_adopt = [0usize; NUM_PHASE];
        for t in 0..n {
            for p in 0..NUM_PHASE {
                phase_iter[p] += self.iteration[t][p];
                phase_adopt[p] += self.adopted[t][p];
            }
        }
        eprintln!("");
        eprintln!("per-phase totals:");
        {
            use std::fmt::Write as _;
            let mut header = String::new();
            let _ = write!(&mut header, "{:>6} |", "phase");
            for p in 0..NUM_PHASE {
                let _ = write!(&mut header, " {:>8}", format!("p{}", p));
            }
            let _ = write!(&mut header, " | {:>8}", "avg");
            eprintln!("{}", header);
        }
        let sep_len = 6 + 3 + NUM_PHASE * 9 + 3 + 8;
        eprintln!("{}", "-".repeat(sep_len));
        let mut avg_it = 0usize;
        let mut avg_ad = 0usize;
        let mut rates = vec![0.0f64; NUM_PHASE];
        for p in 0..NUM_PHASE {
            let it = phase_iter[p];
            let ad = phase_adopt[p];
            avg_it += it;
            avg_ad += ad;
            rates[p] = if it == 0 {
                0.0
            } else {
                (ad as f64) / (it as f64) * 100.0
            };
        }
        let avg_rate = if avg_it == 0 {
            0.0
        } else {
            (avg_ad as f64) / (avg_it as f64) * 100.0
        };
        use std::fmt::Write as _;
        let mut line = String::new();
        let _ = write!(&mut line, "{:>6} |", "total");
        for p in 0..NUM_PHASE {
            let _ = write!(&mut line, " {:>8.2}", rates[p]);
        }
        let _ = write!(&mut line, " | {:>8.2}", avg_rate);
        eprintln!("{}", line);
    }
}

impl<R: Random>
    AnnealerSchedulerWithStatistics<
        AnnealingCriterion,
        ExpTemperatureScheduler,
        SecondProgressScheduler,
        R,
    >
{
    pub fn with_seconds(
        start_temp: f64,
        end_temp: f64,
        seconds: f64,
        is_maximize: bool,
        n_types: usize,
        rnd: R,
    ) -> Self {
        let inner =
            AnnealerScheduler::with_seconds(start_temp, end_temp, seconds, is_maximize, rnd);
        Self::new(inner, n_types)
    }
}

impl<R: Random>
    AnnealerSchedulerWithStatistics<
        AnnealingCriterion,
        ExpTemperatureScheduler,
        IterationProgressScheduler,
        R,
    >
{
    pub fn with_iterations(
        start_temp: f64,
        end_temp: f64,
        iteration: usize,
        is_maximize: bool,
        n_types: usize,
        rnd: R,
    ) -> Self {
        let inner =
            AnnealerScheduler::with_iterations(start_temp, end_temp, iteration, is_maximize, rnd);
        Self::new(inner, n_types)
    }
}

#[cfg(test)]
mod tests {
    use crate::{annealer::prelude::AnnealerScheduler, utils::random::XorShift32};

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

    #[test]
    fn test_annealer_scheduler_with_statistics() {
        const ITERATIONS: usize = 50;
        const N_TYPES: usize = 3;
        let mut scheduler =
            crate::annealer::scheduler::AnnealerSchedulerWithStatistics::with_iterations(
                1e0,
                1e-4,
                ITERATIONS,
                true,
                N_TYPES,
                XorShift32::new(24),
            );
        let mut total_iterations = 0;
        while scheduler.to_next_iter() {
            scheduler.adopt(
                total_iterations % N_TYPES,
                1.,
                2. * (total_iterations % 2) as f64,
            );
            total_iterations += 1;
        }

        let recorded_iterations: usize = scheduler
            .get_iteration()
            .iter()
            .map(|row| row.iter().sum::<usize>())
            .sum();
        scheduler.eprint_statistics();
        assert_eq!(total_iterations, recorded_iterations);
    }
}
