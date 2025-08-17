/*
TODO:
- 統計出力
- 重みの自動計算
- ロガー

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

use crate::annealer::components::Mutator;
use crate::annealer::types::{
    Criterion, NeighborGenerator, NeighborHandler, NeighborType, Scheduler, State,
};
use crate::utils::rnd::Rnd;

pub struct AnnealerConfig<C, S>
where
    C: Criterion,
    S: Scheduler,
{
    pub iteration: usize,
    pub criterion: C,
    pub scheduler: S,
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
    mutator: Mutator<G, N>,
    config: AnnealerConfig<C, S>,
    rnd: Rnd,
}

impl<G, N, C, S> Annealer<G, N, C, S>
where
    G: NeighborGenerator<N>,
    N: NeighborType,
    C: Criterion,
    S: Scheduler,
{
    pub fn new(
        state: <N::H as NeighborHandler>::State,
        env: <N::H as NeighborHandler>::Env,
        mutator: Mutator<G, N>,
        config: AnnealerConfig<C, S>,
        rnd: Rnd,
    ) -> Annealer<G, N, C, S> {
        Annealer {
            state,
            env,
            mutator,
            config,
            rnd,
        }
    }

    pub fn run(&mut self) {
        let mut cur_score = self.state.calc_score(&self.env);
        for t in 0..self.config.iteration {
            let progress = t as f64 / self.config.iteration as f64;
            self.mutator
                .mutate(&mut self.state, &self.env, progress, &mut self.rnd);

            let new_score = self.state.get_score(&self.env);

            let cur_temp = self.config.scheduler.get_temp(progress);

            if self
                .config
                .criterion
                .adopt(cur_score, new_score, cur_temp, progress, &mut self.rnd)
            {
                cur_score = new_score;
            } else {
                self.mutator
                    .revert(&mut self.state, &self.env, &mut self.rnd);
            }

            if t % self.config.log_interval == 0 {
                eprintln!("[{:5}] {} -> {}", t, cur_score, new_score);
            }
        }
    }
}
