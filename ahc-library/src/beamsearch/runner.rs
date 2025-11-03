use std::collections::HashSet;

use crate::utils::time;

pub trait BeamWidthPolicy {
    fn suggest_width(&self, time_progress: f64, cur_turn: usize, end_turn: usize) -> usize;
    fn start_unit(&mut self) {}
    fn end_unit(&mut self) {}
}

/// Delegate trait for beam search
pub trait BeamSearchDelegate<S, M> {
    /// 評価関数（大きい方が良い）
    fn evaluate(&self, state: &mut S, material: &M) -> f64;
    /// 状態遷移関数
    fn transfer(&self, state: &S, material: &M) -> S;
    /// ハッシュ関数
    fn hash(&self, state: &S, material: &M) -> u64;
}

enum BeamSearchRunnerStatus {
    NotStarted,
    InProgress(f64, usize),
    Finished,
}

/// Runner used for beam search process
///
/// Usage:
/// ```ignore
/// let mut states = ...;
///
/// let width_policy = FixedBeamWidthPolicy::new(10);
/// let mut runner = BeamSearchRunner::new(1.5, 20, width_policy);
///
/// while runner.to_next_turn() {
///     let mut transfer_materials = vec![];
///     for (state_i, state) in states.iter().enumerate() {
///         runner.start_unit();
///
///         // generate transfer materials
///
///         runner.end_unit();
///     }
///     states = runner.next_states(states, transfer_materials, &delegate);
/// }
/// ```
pub struct BeamSearchRunner<W>
where
    W: BeamWidthPolicy,
{
    status: BeamSearchRunnerStatus,
    desired_duration_sec: f64,
    end_turn: usize,
    width_policy: W,
}

impl<W> BeamSearchRunner<W>
where
    W: BeamWidthPolicy,
{
    pub fn new(desired_duration_sec: f64, end_turn: usize, width_policy: W) -> Self {
        BeamSearchRunner {
            status: BeamSearchRunnerStatus::NotStarted,
            desired_duration_sec,
            end_turn,
            width_policy,
        }
    }

    pub fn run<S, M, G, D>(
        &mut self,
        states: Vec<S>,
        gen_transfer_materials: G,
        delegate: &D,
    ) -> Vec<S>
    where
        G: Fn(usize, &S) -> Vec<M>,
        D: BeamSearchDelegate<S, M>,
    {
        let mut states = states;

        while self.to_next_turn() {
            let mut transfer_materials = vec![];
            for (state_i, state) in states.iter().enumerate() {
                self.start_unit();
                transfer_materials.extend(
                    gen_transfer_materials(state_i, state)
                        .into_iter()
                        .map(|m| (state_i, m)),
                );
                self.end_unit();
            }

            states = self.next_states(states, transfer_materials, delegate);
        }

        states
    }

    /// 次の状態をビーム幅に基づいて選択して返す
    ///
    /// S: 状態の型
    /// M: 状態遷移に必要な素材の型
    /// D: BeamSearchDelegate
    pub fn next_states<S, M, D>(
        &self,
        mut states: Vec<S>,
        transfer_materials: Vec<(usize, M)>,
        delegate: &D,
    ) -> Vec<S>
    where
        D: BeamSearchDelegate<S, M>,
    {
        let (time_progress, cur_turn) = self.get_progress();
        let beam_width = self
            .width_policy
            .suggest_width(time_progress, cur_turn, self.end_turn);

        let mut cands = vec![];
        for (i, (state_i, material)) in transfer_materials.iter().enumerate() {
            let score = delegate.evaluate(&mut states[*state_i], material);
            cands.push((score, i));
        }

        let mut set = HashSet::new();
        cands.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        let cands = cands
            .into_iter()
            .map(|(_, idx)| idx)
            .filter(|&idx| {
                let (state_i, material) = &transfer_materials[idx];
                let h = delegate.hash(&states[*state_i], material);
                set.insert(h)
            })
            .take(beam_width)
            .collect::<Vec<usize>>();

        let mut next_states = Vec::new();
        for idx in cands {
            let (state_i, material) = &transfer_materials[idx];
            let new_state = delegate.transfer(&states[*state_i], material);
            next_states.push(new_state);
        }

        next_states
    }

    pub fn to_next_turn(&mut self) -> bool {
        self.status = match self.status {
            BeamSearchRunnerStatus::NotStarted => {
                BeamSearchRunnerStatus::InProgress(time::elapsed_seconds(), 0)
            }
            BeamSearchRunnerStatus::InProgress(start_time, turn) => {
                if turn + 1 >= self.end_turn {
                    BeamSearchRunnerStatus::Finished
                } else {
                    BeamSearchRunnerStatus::InProgress(start_time, turn + 1)
                }
            }
            BeamSearchRunnerStatus::Finished => BeamSearchRunnerStatus::Finished,
        };
        matches!(self.status, BeamSearchRunnerStatus::InProgress(_, _))
    }

    pub fn get_progress(&self) -> (f64, usize) {
        let (start_time, cur_turn) = match self.status {
            BeamSearchRunnerStatus::NotStarted => {
                panic!("BeamSearchRunner must be started before get_progress");
            }
            BeamSearchRunnerStatus::InProgress(start_time, turn) => (start_time, turn),
            BeamSearchRunnerStatus::Finished => {
                return (1.0, self.end_turn);
            }
        };
        let elapsed_sec = time::elapsed_seconds() - start_time;
        let time_progress = (elapsed_sec / self.desired_duration_sec).min(1.);

        (time_progress, cur_turn)
    }

    pub fn start_unit(&mut self) {
        self.width_policy.start_unit();
    }

    pub fn end_unit(&mut self) {
        self.width_policy.end_unit();
    }
}

#[cfg(test)]
mod tests {
    use crate::beamsearch::{
        components::FixedBeamWidthPolicy,
        runner::{BeamSearchDelegate, BeamSearchRunner},
    };

    #[test]
    fn test_beam_search_runner_run() {
        struct State {
            a: i64,
        }
        struct Material {
            d: i64,
        }

        let global_c = 10.1;
        struct Delegate {
            c: f64,
        }
        impl BeamSearchDelegate<State, Material> for Delegate {
            fn evaluate(&self, state: &mut State, material: &Material) -> f64 {
                -((state.a + material.d) as f64 - self.c).powf(2.)
            }

            fn transfer(&self, state: &State, material: &Material) -> State {
                State {
                    a: state.a + material.d,
                }
            }

            fn hash(&self, state: &State, material: &Material) -> u64 {
                (state.a + material.d) as u64
            }
        }

        let states = vec![State { a: 1 }];
        let policy = FixedBeamWidthPolicy::new(5);
        let delegate = Delegate { c: global_c };
        let mut runner = BeamSearchRunner::new(5., 3, policy);
        let states = runner.run(
            states,
            |_, _| (1..=5).map(|d| Material { d }).collect(),
            &delegate,
        );

        assert_eq!(states.len(), 5);
        assert_eq!(states[0].a, 10);
        assert_eq!(states[1].a, 11);
        assert_eq!(states[2].a, 9);
        assert_eq!(states[3].a, 12);
        assert_eq!(states[4].a, 8);
    }
}
