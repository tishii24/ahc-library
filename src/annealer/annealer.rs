/*
TODO:
- Op
- ベスト解出力
- キック
- デバッグモード
    - 差分計算のチェック
    - ロガーの無効化
- ベスト解からのやり直し
- 時間の取得間隔を設定
- ロガー
    - スコアの遷移

- 近傍確率の時間に応じた調整
- 近傍ごとの受諾確率調整
- [焼きなまし法での評価関数の打ち切り](https://qiita.com/not522/items/cd20b87157d15850d31c)

- 温度自動調整
*/

use std::collections::BTreeSet;

use crate::annealer::types::{
    Criterion, NeighborGenerator, NeighborHandler, NeighborType, ProgressScheduler, State,
    TemperatureScheduler,
};
use crate::utils::rnd::Rnd;

pub struct AnnealerConfig {}

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
    pub log_store: AnnealingLogStore,
    mutator: Mutator<G, N>,
    criterion: C,
    temperature: T,
    progress: P,
    rnd: Rnd,
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
        progress: P,
        criterion: C,
        temperature: T,
        config: AnnealerConfig,
    ) -> Annealer<G, N, C, T, P> {
        Annealer {
            state,
            env,
            log_store: AnnealingLogStore::new(),
            mutator,
            progress,
            criterion,
            temperature,
            rnd: Rnd::new(24),
            config,
        }
    }

    pub fn run(&mut self) {
        self.progress.start();

        loop {
            let progress = self.progress.get_progress();
            if progress >= 1. {
                break;
            }

            let step_log = self.step(progress);
            self.log_store.send_log(step_log);
            self.progress.step();
        }
    }

    fn cur_step(&self) -> usize {
        self.log_store.logs.len()
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
        let cur_temp = self.temperature.get_temp(progress);
        let adopt = self
            .criterion
            .adopt(cur_score, new_score, cur_temp, progress, &mut self.rnd);

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

pub struct AnnealingLogStore {
    logs: Vec<StepLog>,
}

impl AnnealingLogStore {
    fn new() -> Self {
        AnnealingLogStore { logs: Vec::new() }
    }

    fn send_log(&mut self, step_log: StepLog) {
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
        eprintln!("================== annealing results ==================");
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
            eprintln!(
                "  {:<15}: {:5}/{:<5} ({:5.2}%, Δ={:8.2})",
                tag,
                adopted_steps,
                tag_steps,
                adopted_steps as f64 / tag_steps as f64 * 100.0,
                delta_mean,
            );
        }
        eprintln!("=======================================================");
    }
}
