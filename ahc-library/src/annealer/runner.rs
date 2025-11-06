use std::collections::BTreeSet;

use crate::annealer::scheduler::AnnealerScheduler;
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
}

#[allow(clippy::type_complexity)]
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
    #[allow(clippy::type_complexity)]
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
        for callback in &mut self.callbacks {
            callback.on_start(&mut self.state, &self.env);
        }

        let mut cur_step = 0;
        while self.scheduler.to_next_iter() {
            let progress = self.scheduler.get_progress();
            for callback in &mut self.callbacks {
                callback.on_before_step(cur_step, progress, &mut self.state, &self.env);
            }

            let step_log = self.step(progress);

            if self.config.mode != AnnealerMode::Release {
                self.logger.send(step_log);
            }

            for callback in &mut self.callbacks {
                callback.on_after_step(cur_step, progress, &mut self.state, &self.env);
            }

            cur_step += 1;
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
