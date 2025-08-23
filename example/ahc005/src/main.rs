use std::collections::HashMap;

use ahc_library::annealer::annealer::AnnealerMode;
use ahc_library::annealer::components::callback::{
    RestoreBestStateCallback, ReturnBestStateCallback,
};
use ahc_library::annealer::prelude::*;
use ahc_library::annealer::types::Callback;
use ahc_library::utils::index_set::IndexSet;
use ahc_library::utils::rnd::Rnd;
use proconio::input;

const D: [(usize, usize); 4] = [(!0, 0), (0, 1), (1, 0), (0, !0)];

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
    edges: Vec<Vec<(usize, i64, (usize, usize))>>,
    vertices: Vec<(usize, usize)>,
    covers: Vec<Vec<usize>>,
    num_targets: usize,
}

impl Env for EnvImpl {}

impl EnvImpl {
    fn from_input(input: &Input) -> EnvImpl {
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
                if input.c[i][j].is_none() {
                    continue;
                }
                let mut c = 0;
                let mut tc = 0;
                for d in 0..2 {
                    let nxt_1 = (i + D[d].0, j + D[d].1);
                    let nxt_2 = (i + D[d + 2].0, j + D[d + 2].1);
                    let a = nxt_1.0 < input.n
                        && nxt_1.1 < input.n
                        && input.c[nxt_1.0][nxt_1.1].is_some();
                    let b = nxt_2.0 < input.n
                        && nxt_2.1 < input.n
                        && input.c[nxt_2.0][nxt_2.1].is_some();

                    if a || b {
                        c += 1;
                    }
                    if a ^ b {
                        tc += 1;
                    }
                }
                if c == 2 || tc == 1 {
                    vertices_map[i][j] = Some(vertices.len());
                    vertices.push((i, j));
                }
            }
        }

        #[allow(non_snake_case)]
        let V = vertices.len();

        let mut edges = vec![vec![]; V];
        let mut covers: Vec<Vec<usize>> = (0..vertices.len()).map(|_| vec![]).collect();
        let mut path_map = HashMap::new();

        for i in 0..input.n {
            for j in 0..input.n {
                let Some(v_idx) = vertices_map[i][j] else {
                    continue;
                };
                for (di, d) in D.iter().enumerate() {
                    let mut cum_cost = 0;
                    let mut cur = (i, j);
                    let mut c = 0;
                    let mut last_coor = cur;
                    loop {
                        c += 1;
                        let nxt = (cur.0 + d.0, cur.1 + d.1);
                        if nxt.0 >= input.n || nxt.1 >= input.n || input.c[nxt.0][nxt.1].is_none() {
                            break;
                        }
                        cum_cost += input.c[nxt.0][nxt.1].unwrap();

                        if let Some(u_idx) = vertices_map[nxt.0][nxt.1] {
                            // 途中にパスがあれば、それを追加する
                            if nxt.0.abs_diff(last_coor.0) + nxt.1.abs_diff(last_coor.1) > 1 {
                                let key = (
                                    (nxt.0.min(last_coor.0), nxt.1.min(last_coor.1)),
                                    (nxt.0.max(last_coor.0), nxt.1.max(last_coor.1)),
                                );
                                if let Some(i) = path_map.get(&key) {
                                    covers[v_idx].push(V + *i);
                                } else {
                                    covers[v_idx].push(V + path_map.len());
                                    path_map.insert(key, path_map.len());
                                }
                            }

                            covers[v_idx].push(u_idx);
                            edges[v_idx].push((u_idx, cum_cost as i64, (di, c)));
                            last_coor = nxt;
                        }
                        cur = nxt;
                    }
                }
            }
        }

        let mut dist = vec![vec![1 << 30; V]; V];
        for v in 0..V {
            dist[v][v] = 0;
            for &(u, cost, _) in edges[v].iter() {
                dist[v][u] = cost;
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
            edges,
            vertices,
            covers,
            num_targets: V + path_map.len(),
        }
    }
}

#[derive(Clone)]
struct StateImpl {
    score: i64,
    path: Vec<usize>,
    counter: Vec<usize>,
    unvisited_count: usize,
    unused: IndexSet,
}

impl State<EnvImpl> for StateImpl {
    fn get_score(&mut self, _: &EnvImpl, _: f64) -> f64 {
        self.score as f64 + self.unvisited_count as f64 * 1e6
    }
}

impl StateImpl {
    fn calc_score(&mut self, env: &EnvImpl, _: f64) -> f64 {
        self.counter = vec![0; env.num_targets];
        self.score = 0;

        for i in 0..self.path.len() - 1 {
            self.score += env.dist[self.path[i]][self.path[i + 1]];
        }

        for &v in self.path.iter() {
            for &u in env.covers[v].iter() {
                self.counter[u] += 1;
            }
        }

        self.unvisited_count = 0;
        for &e in self.counter.iter() {
            if e == 0 {
                self.unvisited_count += 1;
            }
        }

        self.score as f64 + self.unvisited_count as f64 * 1e6
    }

    fn add_v(&mut self, v: usize, env: &EnvImpl) {
        for &u in env.covers[v].iter() {
            if self.counter[u] == 0 {
                self.unvisited_count -= 1;
            }
            self.counter[u] += 1;
        }
    }

    fn remove_v(&mut self, v: usize, env: &EnvImpl) {
        for &u in env.covers[v].iter() {
            self.counter[u] -= 1;
            if self.counter[u] == 0 {
                self.unvisited_count += 1;
            }
        }
    }
}

struct NeighborSwap {
    ijs: Option<(usize, usize, i64)>,
}

impl NeighborSwap {
    fn setup() -> NeighborSwap {
        NeighborSwap { ijs: None }
    }

    fn apply(&mut self, state: &mut StateImpl, env: &EnvImpl, rnd: &mut Rnd) -> bool {
        let (i, j) = (
            rnd.gen_range(1, state.path.len() - 1),
            rnd.gen_range(1, state.path.len() - 1),
        );
        if i == j {
            return false;
        }
        let (i, j) = (i.min(j), i.max(j));

        let prev_score = if i + 1 == j {
            env.dist[state.path[i - 1]][state.path[i]]
                + env.dist[state.path[i]][state.path[j]]
                + env.dist[state.path[j]][state.path[j + 1]]
        } else {
            env.dist[state.path[i - 1]][state.path[i]]
                + env.dist[state.path[i]][state.path[i + 1]]
                + env.dist[state.path[j - 1]][state.path[j]]
                + env.dist[state.path[j]][state.path[j + 1]]
        };

        state.path.swap(i, j);

        let new_score = if i + 1 == j {
            env.dist[state.path[i - 1]][state.path[i]]
                + env.dist[state.path[i]][state.path[j]]
                + env.dist[state.path[j]][state.path[j + 1]]
        } else {
            env.dist[state.path[i - 1]][state.path[i]]
                + env.dist[state.path[i]][state.path[i + 1]]
                + env.dist[state.path[j - 1]][state.path[j]]
                + env.dist[state.path[j]][state.path[j + 1]]
        };

        let s = new_score - prev_score;
        state.score += s;

        self.ijs = Some((i, j, s));

        true
    }

    fn revert(&mut self, state: &mut StateImpl, _: &EnvImpl, _: &mut Rnd) {
        let (i, j, s) = self.ijs.unwrap();
        state.path.swap(i, j);
        state.score -= s;
    }

    fn tag(&self) -> &'static str {
        "Swap"
    }
}

struct NeighborDrop {
    ivs: Option<(usize, usize, i64)>,
}

impl NeighborDrop {
    fn setup() -> NeighborDrop {
        NeighborDrop { ivs: None }
    }

    fn apply(&mut self, state: &mut StateImpl, env: &EnvImpl, rnd: &mut Rnd) -> bool {
        if state.path.len() <= 2 {
            return false;
        }
        let i = rnd.gen_range(1, state.path.len() - 1);

        let prev_score =
            env.dist[state.path[i - 1]][state.path[i]] + env.dist[state.path[i]][state.path[i + 1]];
        let new_score = env.dist[state.path[i - 1]][state.path[i + 1]];
        let s = new_score - prev_score;

        let v = state.path.remove(i);
        state.remove_v(v, env);
        state.unused.add(v);
        state.score += s;
        self.ivs = Some((i, v, s));

        true
    }

    fn revert(&mut self, state: &mut StateImpl, env: &EnvImpl, _: &mut Rnd) {
        let (i, v, s) = self.ivs.unwrap();
        state.unused.remove(v);
        state.add_v(v, env);
        state.path.insert(i, v);
        state.score -= s;
    }

    fn tag(&self) -> &'static str {
        "Drop"
    }
}

struct NeighborInsert {
    is: Option<(usize, i64)>,
}

impl NeighborInsert {
    fn setup() -> NeighborInsert {
        NeighborInsert { is: None }
    }

    fn apply(&mut self, state: &mut StateImpl, env: &EnvImpl, rnd: &mut Rnd) -> bool {
        if state.unused.size() == 0 {
            return false;
        }

        let i = rnd.gen_range(1, state.path.len());
        let v = state.unused.get_random(rnd);

        let prev_score = env.dist[state.path[i - 1]][state.path[i]];
        let new_score = env.dist[state.path[i - 1]][v] + env.dist[v][state.path[i]];
        let s = new_score - prev_score;

        state.path.insert(i, v);
        state.unused.remove(v);
        state.add_v(v, env);
        state.score += s;
        self.is = Some((i, s));

        true
    }

    fn revert(&mut self, state: &mut StateImpl, env: &EnvImpl, _: &mut Rnd) {
        let (i, s) = self.is.unwrap();
        let v = state.path.remove(i);
        state.unused.add(v);
        state.remove_v(v, env);
        state.score -= s;
    }

    fn tag(&self) -> &'static str {
        "Insert"
    }
}

neighbor_impl!(
    StateImpl,
    EnvImpl,
    NeighborSwap,
    NeighborDrop,
    NeighborInsert
);

fn init_state(env: &EnvImpl) -> StateImpl {
    let mut path: Vec<usize> = (0..env.vertices.len()).collect();
    path.push(env.start_idx);

    let mut state = StateImpl {
        score: 0,
        path,
        counter: vec![0; env.num_targets],
        unvisited_count: 0,
        unused: IndexSet::empty(env.vertices.len()),
    };
    state.calc_score(env, 0.);
    state
}

fn output(state: &StateImpl, env: &EnvImpl) {
    let mut cmds = vec![];
    for i in 0..state.path.len() - 1 {
        let (v, u) = (state.path[i], state.path[i + 1]);
        let d = env.dist[v][u];
        let mut cur = v;
        while cur != u {
            for &(nxt, _, (di, c)) in env.edges[cur].iter() {
                if env.dist[v][cur] + env.dist[cur][nxt] + env.dist[nxt][u] == d {
                    cmds.push((di, c));

                    cur = nxt;
                    break;
                }
            }
        }
    }

    for (di, c) in cmds {
        // [(!0, 0), (0, 1), (1, 0), (0, !0)];
        let e = match di {
            0 => 'U',
            1 => 'R',
            2 => 'D',
            _ => 'L',
        };
        for _ in 0..c {
            print!("{}", e);
        }
    }
    println!();
}

fn main() {
    let input = Input::read_input();
    let env = EnvImpl::from_input(&input);
    let state = init_state(&env);
    let generator = WeightedNeighborGenerator::new(vec![
        (Neighbor::NeighborSwap, 0.8),
        (Neighbor::NeighborDrop, 0.8),
        (Neighbor::NeighborInsert, 0.4),
    ]);
    let mutator = Mutator::new(generator);
    let callbacks: Vec<Box<dyn Callback<StateImpl, EnvImpl>>> = vec![
        // Box::new(RestoreBestStateCallback::new(100_000, false)),
        // Box::new(ReturnBestStateCallback::new(false)),
    ];
    let mut annealer = Annealer::new(
        state,
        env,
        mutator,
        SecondProgressScheduler::new(1.9),
        AnnealingCriterion::new(false),
        ExpTemperatureScheduler::new(1e2, 1e-1),
        callbacks,
        AnnealerConfig {
            mode: AnnealerMode::Release,
        },
    );
    annealer.run();

    let (state, env, logger) = (annealer.state, annealer.env, annealer.log_store);
    logger.print();

    output(&state, &env);
}
