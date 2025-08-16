/*
TODO:
- デバッグモード
- 温度自動調整
- 統計出力
- 近傍確率の時間に応じた調整
- 近傍ごとの受諾確率調整
- ベスト解出力
- 温度スケジュール
- 重みの自動計算
- 乱数生成
- 時間の取得間隔を設定
- キック
- ベスト解からのやり直し
*/

use crate::annealer::components::Mutator;
use crate::annealer::types::{NeighborGenerator, NeighborHandler, NeighborType, State};

pub struct AnnealerConfig {
    pub start_temp: f64,
    pub end_temp: f64,
    pub iteration: usize,
    pub is_maximize: bool,
    pub log_interval: usize,
}

pub struct Annealer<G, N>
where
    G: NeighborGenerator<N>,
    N: NeighborType,
{
    pub state: <N::H as NeighborHandler>::State,
    pub env: <N::H as NeighborHandler>::Env,
    mutator: Mutator<G, N>,
    pub config: AnnealerConfig,
}

impl<G, N> Annealer<G, N>
where
    G: NeighborGenerator<N>,
    N: NeighborType,
{
    pub fn new(
        state: <N::H as NeighborHandler>::State,
        env: <N::H as NeighborHandler>::Env,
        mutator: Mutator<G, N>,
        config: AnnealerConfig,
    ) -> Annealer<G, N> {
        Annealer {
            state,
            env,
            mutator,
            config,
        }
    }

    pub fn run(&mut self) {
        let mut cur_score = self.state.calc_score(&self.env);
        for t in 0..self.config.iteration {
            let progress = t as f64 / self.config.iteration as f64;
            self.mutator.mutate(&mut self.state, &self.env, progress);

            let new_score = self.state.calc_score(&self.env);
            if self.adopt(cur_score, new_score, progress) {
                cur_score = new_score;
            } else {
                self.mutator.revert(&mut self.state, &self.env);
            }

            if t % self.config.log_interval == 0 {
                eprintln!("[{:5}] {} -> {}", t, cur_score, new_score);
            }
        }
    }

    fn adopt(&self, cur_score: f64, new_score: f64, progress: f64) -> bool {
        let cur_temp =
            self.config.start_temp.powf(1. - progress) * self.config.end_temp.powf(progress);
        new_score < cur_score
    }

    fn register_result(&self, result: bool, improved: bool, score_delta: f32) {
        todo!()
    }
}
