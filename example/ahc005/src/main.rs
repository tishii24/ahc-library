use ahc_library::annealer::prelude::*;
use ahc_library::utils::rnd::Rnd;
use proconio::input;

struct Input {
    n: usize,
    s: (usize, usize),
    c: Vec<Vec<Option<u32>>>,
}

impl Input {
    fn read_input() -> Input {
        input! {
            n: usize,
            s: (usize, usize),
            c: [String; n],
        }
        let c = c
            .into_iter()
            .map(|c| c.chars().into_iter().map(|e| e.to_digit(10)).collect())
            .collect();
        Input { n, s, c }
    }
}

struct EnvImpl {
    start_idx: usize,
    dist: Vec<Vec<i64>>,
    vertices: Vec<(usize, usize)>,
}

impl Env for EnvImpl {}

impl EnvImpl {
    fn from_input(input: &Input) -> EnvImpl {
        const D: [(usize, usize); 4] = [(!0, 0), (0, 1), (1, 0), (0, !0)];

        let mut vertices = vec![];
        let mut vertices_map = vec![vec![None; input.n]; input.n];
        let start_idx = 0;

        vertices_map[input.s.0][input.s.1] = Some(start_idx);
        vertices.push(input.s);

        for i in 0..input.n {
            for j in 0..input.n {
                if (i, j) == input.s {
                    continue;
                }
                let mut c = 0;
                for d in 0..2 {
                    for e in [D[d], D[d + 2]] {
                        let nxt = (i + e.0, j + e.1);
                        if nxt.0 >= input.n || nxt.1 >= input.n {
                            continue;
                        }
                        if input.c[i][j].is_some() {
                            c += 1;
                        }
                    }
                }
                if c == 2 {
                    vertices_map[i][j] = Some(vertices.len());
                    vertices.push((i, j));
                }
            }
        }

        #[allow(non_snake_case)]
        let V = vertices.len();

        let mut edges = vec![vec![]; V];
        for i in 0..input.n {
            for j in 0..input.n {
                let Some(v_idx) = vertices_map[i][j] else {
                    continue;
                };
                for d in D {
                    let mut cum_cost = 0;
                    let mut cur = (i, j);
                    loop {
                        let nxt = (cur.0 + d.0, cur.1 + d.1);
                        if nxt.0 >= input.n || nxt.1 >= input.n || input.c[nxt.0][nxt.1].is_none() {
                            break;
                        }
                        cum_cost += input.c[nxt.0][nxt.1].unwrap();
                        if let Some(u_idx) = vertices_map[nxt.0][nxt.1] {
                            edges[v_idx].push((u_idx, cum_cost as i64));
                        }
                        cur = nxt;
                    }
                }
            }
        }

        let mut dist = vec![vec![1 << 30; V]; V];
        for v in 0..V {
            for &(u, cost) in edges[v].iter() {
                dist[v][u] = cost;
                dist[u][v] = cost;
            }
        }

        for k in 0..V {
            for i in 0..V {
                for j in 0..V {
                    dist[i][j] = dist[i][j].min(dist[i][k] + dist[k][j]);
                }
            }
        }

        EnvImpl {
            start_idx,
            dist,
            vertices,
        }
    }
}

struct StateImpl {
    path: Vec<usize>,
}

impl State<EnvImpl> for StateImpl {
    fn calc_score(&mut self, env: &EnvImpl, progress: f64) -> f64 {
        0.0
    }
}

struct NeighborA;

impl NeighborA {
    fn generate() -> NeighborA {
        NeighborA {}
    }

    fn apply(&mut self, state: &mut StateImpl, env: &EnvImpl, rnd: &mut Rnd) -> bool {
        true
    }

    fn revert(&mut self, state: &mut StateImpl, env: &EnvImpl, _: &mut Rnd) {}

    fn tag(&self) -> &'static str {
        "A"
    }
}

neighbor_impl!(StateImpl, EnvImpl, NeighborA);

fn init_state(env: &EnvImpl) -> StateImpl {
    StateImpl {
        path: (0..env.vertices.len()).collect(),
    }
}

fn main() {
    let input = Input::read_input();
    let env = EnvImpl::from_input(&input);
    let state = init_state(&env);
    let generator = WeightedNeighborGenerator::new(vec![(Neighbor::NeighborA, 0.8)]);
    let mutator = Mutator::new(generator);
    let mut annealer = Annealer::new(
        state,
        env,
        mutator,
        SecondProgressScheduler::new(0.1),
        AnnealingCriterion::new(false),
        ExpTemperatureScheduler::new(1e0, 1e-4),
        AnnealerConfig {},
    );
    annealer.run();
}
