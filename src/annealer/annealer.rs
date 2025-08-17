/*
TODO:
- ロガー
    - スコアの遷移

- Op
- ベスト解出力
- キック
- デバッグモード
    - 差分計算のチェック
    - ロガーの無効化
- ベスト解からのやり直し
- 時間の取得間隔を設定

- 温度自動調整
- 近傍確率の時間に応じた調整
- 近傍ごとの受諾確率調整
- [焼きなまし法での評価関数の打ち切り](https://qiita.com/not522/items/cd20b87157d15850d31c)
*/

use std::collections::HashSet;

use crate::annealer::components::Mutator;
use crate::annealer::types::{
    Criterion, NeighborGenerator, NeighborHandler, NeighborType, Scheduler, State,
};
use crate::utils::rnd::Rnd;

pub struct AnnealerConfig {
    pub iteration: usize,
    pub log_interval: usize,
}

pub struct Annealer<G, N, C, S>
where
    G: NeighborGenerator<N>,
    N: NeighborType,
    C: Criterion,
    S: Scheduler,
{
    pub state: <N::H as NeighborHandler>::State,
    pub env: <N::H as NeighborHandler>::Env,
    pub criterion: C,
    pub scheduler: S,
    config: AnnealerConfig,
    pub log_store: LogStore,
    mutator: Mutator<G, N>,
    rnd: Rnd,
    cur_score: f64,
}

impl<G, N, C, S> Annealer<G, N, C, S>
where
    G: NeighborGenerator<N>,
    N: NeighborType,
    C: Criterion,
    S: Scheduler,
{
    pub fn new(
        mut state: <N::H as NeighborHandler>::State,
        env: <N::H as NeighborHandler>::Env,
        mutator: Mutator<G, N>,
        criterion: C,
        scheduler: S,
        config: AnnealerConfig,
        rnd: Rnd,
    ) -> Annealer<G, N, C, S> {
        let cur_score = state.calc_score(&env);
        Annealer {
            state,
            env,
            mutator,
            criterion,
            scheduler,
            config,
            log_store: LogStore::new(),
            rnd,
            cur_score,
        }
    }

    pub fn run(&mut self) {
        for t in 0..self.config.iteration {
            let progress = t as f64 / self.config.iteration as f64;
            let step_log = self.step(progress);
            self.log_store.send_log(step_log);
        }
    }

    fn step(&mut self, progress: f64) -> StepLog {
        let (successed, tag) =
            self.mutator
                .mutate(&mut self.state, &self.env, progress, &mut self.rnd);

        if !successed {
            return StepLog {
                score: self.cur_score,
                adopt: false,
                valid: false,
                tag,
                score_delta: 0.,
            };
        }

        let new_score = self.state.get_score(&self.env);
        let score_delta = new_score - self.cur_score;
        let cur_temp = self.scheduler.get_temp(progress);
        let adopt =
            self.criterion
                .adopt(self.cur_score, new_score, cur_temp, progress, &mut self.rnd);

        if adopt {
            self.cur_score = new_score;
        } else {
            self.mutator
                .revert(&mut self.state, &self.env, &mut self.rnd);
        }

        StepLog {
            score: self.cur_score,
            adopt,
            valid: true,
            tag,
            score_delta,
        }
    }
}

struct StepLog {
    score: f64,
    adopt: bool,
    valid: bool,
    tag: &'static str,
    score_delta: f64,
}

pub struct LogStore {
    logs: Vec<StepLog>,
}

impl LogStore {
    fn new() -> Self {
        LogStore { logs: Vec::new() }
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
        let mut neighbor_tags = self
            .logs
            .iter()
            .map(|log| log.tag)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        neighbor_tags.sort();

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
