use crate::annealer::types::{NeighborGenerator, NeighborHandler, NeighborType};

pub struct WeightedNeighborGenerator<N>
where
    N: NeighborType,
{
    neighbors: Vec<(N, f32)>,
}

impl<N> WeightedNeighborGenerator<N>
where
    N: NeighborType,
{
    pub fn new(neighbors: Vec<(N, f32)>) -> WeightedNeighborGenerator<N> {
        WeightedNeighborGenerator { neighbors }
    }
}

impl<N> NeighborGenerator<N> for WeightedNeighborGenerator<N>
where
    N: NeighborType,
{
    fn generate(&self, _progress: f64) -> N::H {
        // TODO: improve
        self.neighbors[0].0.generate()
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
    ) {
        let n = self.generator.generate(progress);
        n.apply(state, env);
        self.last_neighbor = Some(n);
    }

    pub fn revert(
        &mut self,
        state: &mut <N::H as NeighborHandler>::State,
        env: &<N::H as NeighborHandler>::Env,
    ) {
        let last_neighbor = self
            .last_neighbor
            .take()
            .expect("expect last neighbor being set before revert");
        last_neighbor.revert(state, env);
    }

    fn get_last_tag(&self) -> Option<&'static str> {
        if let Some(last_neighbor) = &self.last_neighbor {
            Some(last_neighbor.tag())
        } else {
            None
        }
    }
}
