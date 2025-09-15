use std::collections::BTreeSet;

use crate::annealer::prelude::{
    AnnealingCriterion, ExpTemperatureScheduler, IterationProgressScheduler,
    SecondProgressScheduler,
};
use crate::annealer::types::{
    Callback, Criterion, NeighborGenerator, NeighborHandler, NeighborType, ProgressScheduler,
    State, TemperatureScheduler,
};
use crate::utils::random::Rnd;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AnnealerMode {
    Debug,
    Release,
}

pub struct AnnealerConfig {
    pub mode: AnnealerMode,
    pub start_count: usize,
}

/// Scheduler used for annealing process
///
/// Usage:
/// ```ignore
/// let mut scheduler = AnnealerScheduler::default(1e0, 1e-4, 1.0, true);
/// scheduler.start();
/// while scheduler.in_progress() {
///     let cur_score = state.get_score();
///
///     // do something
///
///     let new_score = state.calc_score();
///
///     if scheduler.adopt(cur_score, new_score) {
///         // adopt
///     } else {
///         // revert
///     }
///     scheduler.step();
/// }
/// ```
pub struct AnnealerScheduler<C, T, P>
where
    C: Criterion,
    T: TemperatureScheduler,
    P: ProgressScheduler,
{
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
            criterion,
            temperature_scheduler,
            progress_scheduler,
            rnd: Rnd::new(24),
        }
    }

    pub fn start(&mut self) {
        self.progress_scheduler.start();
    }

    pub fn adopt(&mut self, cur_score: f64, new_score: f64) -> bool {
        let progress = self.get_progress();
        let cur_temp = self.temperature_scheduler.get_temp(progress);
        let adopt = self
            .criterion
            .adopt(cur_score, new_score, cur_temp, progress, &mut self.rnd);
        adopt
    }

    pub fn step(&mut self) {
        self.progress_scheduler.step();
    }

    pub fn get_progress(&self) -> f64 {
        self.progress_scheduler.get_progress()
    }

    pub fn in_progress(&self) -> bool {
        self.progress_scheduler.get_progress() < 1.
    }
}

impl<C, T, P> AnnealerScheduler<C, T, P>
where
    C: Criterion,
    T: TemperatureScheduler,
    P: ProgressScheduler,
{
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

pub struct Annealer<G, N, C, T, P>
where
    G: NeighborGenerator<N>,
    N: NeighborType,
    C: Criterion,
    T: TemperatureScheduler,
    P: ProgressScheduler,
{
    pub state: <N::H as NeighborHandler>::State,
    pub env: <N::H as NeighborHandler>::Env,
    pub logger: AnnealingLogger,
    mutator: Mutator<G, N>,
    scheduler: AnnealerScheduler<C, T, P>,
    rnd: Rnd,
    callbacks:
        Vec<Box<dyn Callback<<N::H as NeighborHandler>::State, <N::H as NeighborHandler>::Env>>>,
    config: AnnealerConfig,
}

impl<G, N, C, T, P> Annealer<G, N, C, T, P>
where
    G: NeighborGenerator<N>,
    N: NeighborType,
    C: Criterion,
    T: TemperatureScheduler,
    P: ProgressScheduler,
{
    pub fn new(
        state: <N::H as NeighborHandler>::State,
        env: <N::H as NeighborHandler>::Env,
        mutator: Mutator<G, N>,
        scheduler: AnnealerScheduler<C, T, P>,
        callbacks: Vec<
            Box<dyn Callback<<N::H as NeighborHandler>::State, <N::H as NeighborHandler>::Env>>,
        >,
        config: AnnealerConfig,
    ) -> Annealer<G, N, C, T, P> {
        Annealer {
            state,
            env,
            logger: AnnealingLogger::new(),
            mutator,
            scheduler,
            rnd: Rnd::new(24),
            callbacks,
            config,
        }
    }

    pub fn run(&mut self) {
        self.scheduler.start();

        for callback in &mut self.callbacks {
            callback.on_start(&mut self.state, &self.env);
        }

        for cur_step in 0.. {
            let progress = self.scheduler.get_progress();
            if progress >= 1. {
                break;
            }

            for callback in &mut self.callbacks {
                callback.on_before_step(cur_step, progress, &mut self.state, &self.env);
            }

            let step_log = self.step(progress);

            if self.config.mode != AnnealerMode::Release {
                self.logger.send(step_log);
            }

            self.scheduler.step();

            for callback in &mut self.callbacks {
                callback.on_after_step(cur_step, progress, &mut self.state, &self.env);
            }
        }

        for callback in &mut self.callbacks {
            callback.on_finish(&mut self.state, &self.env);
        }
    }

    fn step(&mut self, progress: f64) -> StepLog {
        let cur_score = self.state.get_score(&self.env, progress);
        let (successed, tag) =
            self.mutator
                .mutate(&mut self.state, &self.env, progress, &mut self.rnd);

        if !successed {
            return StepLog {
                score: cur_score,
                adopt: false,
                valid: false,
                tag,
                score_delta: 0.,
            };
        }

        let new_score = self.state.get_score(&self.env, progress);
        let score_delta = new_score - cur_score;
        let adopt = self.scheduler.adopt(cur_score, new_score);
        if !adopt {
            self.mutator
                .revert(&mut self.state, &self.env, &mut self.rnd);
        }

        StepLog {
            score: cur_score,
            adopt,
            valid: true,
            tag,
            score_delta,
        }
    }
}

pub struct Mutator<G, N>
where
    G: NeighborGenerator<N>,
    N: NeighborType,
{
    generator: G,
    last_neighbor: Option<N::H>,
}

impl<G, N> Mutator<G, N>
where
    G: NeighborGenerator<N>,
    N: NeighborType,
{
    pub fn new(generator: G) -> Mutator<G, N> {
        Mutator {
            generator,
            last_neighbor: None,
        }
    }

    pub fn mutate(
        &mut self,
        state: &mut <N::H as NeighborHandler>::State,
        env: &<N::H as NeighborHandler>::Env,
        progress: f64,
        rnd: &mut Rnd,
    ) -> (bool, &'static str) {
        let mut n = self.generator.generate(progress, rnd);
        let successed = n.apply(state, env, rnd);
        let tag = n.tag();
        self.last_neighbor = Some(n);
        if !successed {
            return (false, tag);
        }
        (true, tag)
    }

    pub fn revert(
        &mut self,
        state: &mut <N::H as NeighborHandler>::State,
        env: &<N::H as NeighborHandler>::Env,
        rnd: &mut Rnd,
    ) {
        let mut last_neighbor = self
            .last_neighbor
            .take()
            .expect("expect last neighbor being set before revert");
        last_neighbor.revert(state, env, rnd);
    }
}

struct StepLog {
    score: f64,
    adopt: bool,
    valid: bool,
    tag: &'static str,
    score_delta: f64,
}

pub struct AnnealingLogger {
    logs: Vec<StepLog>,
}

impl AnnealingLogger {
    fn new() -> Self {
        AnnealingLogger { logs: Vec::new() }
    }

    fn send(&mut self, step_log: StepLog) {
        self.logs.push(step_log);
    }

    pub fn print(&self) {
        let total_steps = self.logs.len();
        let valid_logs = self.logs.iter().filter(|log| log.valid).collect::<Vec<_>>();
        let valid_steps = valid_logs.len();
        let initial_score = self.logs.first().map_or(0.0, |log| log.score);
        let final_score = self.logs.last().map_or(0.0, |log| log.score);
        let neighbor_tags = self.logs.iter().map(|log| log.tag).collect::<BTreeSet<_>>();

        eprintln!();
        eprintln!("======================== annealing results ========================");
        eprintln!("total steps:   {:8}", total_steps);
        eprintln!(
            "valid steps:   {:8} ({:5.2}%)",
            valid_steps,
            valid_steps as f64 / total_steps as f64 * 100.0
        );
        eprintln!("initial score: {:8}", initial_score);
        eprintln!("final score:   {:8}", final_score);
        eprintln!("neighbors:");
        for tag in neighbor_tags {
            let tag_steps = self.logs.iter().filter(|log| log.tag == tag).count();
            let valid_steps = valid_logs.iter().filter(|log| log.tag == tag).count();
            let adopted_steps = valid_logs
                .iter()
                .filter(|log| log.tag == tag && log.adopt)
                .count();
            let delta_mean = valid_logs
                .iter()
                .filter(|log| log.tag == tag && log.adopt)
                .map(|log| log.score_delta)
                .sum::<f64>()
                / adopted_steps.max(1) as f64;
            let valid_ratio = valid_steps as f64 / tag_steps.max(1) as f64;
            eprintln!(
                "  {:<8}: {:5}/{:<5} (adopt={:6.2}%, valid={:6.2}%, Δ={:8.2})",
                tag,
                adopted_steps,
                tag_steps,
                adopted_steps as f64 / tag_steps as f64 * 100.0,
                valid_ratio,
                delta_mean,
            );
        }
        eprintln!("===================================================================");
    }
}
