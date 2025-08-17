use crate::{
    annealer::types::{
        Criterion, NeighborGenerator, NeighborHandler, NeighborType, ProgressScheduler,
        TemperatureScheduler,
    },
    utils::{rnd::Rnd, time},
};

/// TODO: ちゃんと実装する
pub struct WeightedNeighborGenerator<N>
where
    N: NeighborType,
{
    neighbors: Vec<(N, f64)>,
}

impl<N> WeightedNeighborGenerator<N>
where
    N: NeighborType,
{
    pub fn new(mut neighbors: Vec<(N, f64)>) -> WeightedNeighborGenerator<N> {
        let total_weight: f64 = neighbors.iter().map(|(_, weight)| *weight).sum();
        let mut cum_weight = 0.;
        for (_, weight) in neighbors.iter_mut() {
            assert!(*weight >= 0.0);
            cum_weight += *weight / total_weight;
            *weight = cum_weight;
        }

        WeightedNeighborGenerator { neighbors }
    }
}

impl<N> NeighborGenerator<N> for WeightedNeighborGenerator<N>
where
    N: NeighborType,
{
    fn generate(&self, _progress: f64, rnd: &mut Rnd) -> N::H {
        let p = rnd.nextf();
        for (n, cum_weight) in self.neighbors.iter() {
            if p < *cum_weight {
                return n.generate();
            }
        }
        unreachable!()
    }
}

pub struct HillClimbingCriterion {
    is_maximize: bool,
}

impl HillClimbingCriterion {
    pub fn new(is_maximize: bool) -> Self {
        HillClimbingCriterion { is_maximize }
    }
}

impl Criterion for HillClimbingCriterion {
    fn adopt(&self, cur_score: f64, new_score: f64, _: f64, _: f64, _: &mut Rnd) -> bool {
        if self.is_maximize {
            new_score > cur_score
        } else {
            new_score < cur_score
        }
    }
}

pub struct AnnealingCriterion {
    is_maximize: bool,
}

impl AnnealingCriterion {
    pub fn new(is_maximize: bool) -> Self {
        AnnealingCriterion { is_maximize }
    }
}

impl Criterion for AnnealingCriterion {
    fn adopt(&self, cur_score: f64, new_score: f64, cur_temp: f64, _: f64, rnd: &mut Rnd) -> bool {
        let sign = self.is_maximize as i32 * 2 - 1;
        let score_diff = sign as f64 * (new_score - cur_score);
        if score_diff > 0. {
            return true;
        }
        let prob = (score_diff / cur_temp).exp();
        rnd.nextf() < prob
    }
}

pub struct ExpScheduler {
    start_temp: f64,
    end_temp: f64,
}

impl ExpScheduler {
    pub fn new(start_temp: f64, end_temp: f64) -> Self {
        ExpScheduler {
            start_temp,
            end_temp,
        }
    }
}

impl TemperatureScheduler for ExpScheduler {
    fn get_temp(&self, progress: f64) -> f64 {
        self.start_temp.powf(1. - progress) * self.end_temp.powf(progress)
    }
}

pub struct IterationProgressScheduler {
    iteration: usize,
    cur_step: usize,
}

impl IterationProgressScheduler {
    pub fn new(iteration: usize) -> Self {
        IterationProgressScheduler {
            iteration,
            cur_step: 0,
        }
    }
}

impl ProgressScheduler for IterationProgressScheduler {
    fn step(&mut self) {
        self.cur_step += 1;
    }

    fn get_progress(&self) -> f64 {
        self.cur_step as f64 / self.iteration as f64
    }
}

pub struct SecondProgressScheduler {
    start_time: f64,
    seconds: f64,
}

impl SecondProgressScheduler {
    pub fn new(seconds: f64) -> Self {
        SecondProgressScheduler {
            start_time: 0.0,
            seconds,
        }
    }
}

impl ProgressScheduler for SecondProgressScheduler {
    fn start(&mut self) {
        self.start_time = time::elapsed_seconds();
    }

    fn get_progress(&self) -> f64 {
        (time::elapsed_seconds() - self.start_time) / self.seconds
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
