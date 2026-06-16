use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
};

use crate::utils::fast_clear_array::FastClearArray;

pub const INF_COST: i64 = i64::MAX / 2;

pub trait SearchArray<T: Copy> {
    fn new(n: usize, default: T) -> Self;
    fn get(&mut self, index: usize) -> T;
    fn set(&mut self, index: usize, value: T);
    fn reset(&mut self);
}

impl<T: Copy> SearchArray<T> for FastClearArray<T> {
    fn new(n: usize, default: T) -> Self {
        FastClearArray::new(n, default)
    }

    fn get(&mut self, index: usize) -> T {
        FastClearArray::get(self, index)
    }

    fn set(&mut self, index: usize, value: T) {
        FastClearArray::set(self, index, value);
    }

    fn reset(&mut self) {
        self.clear();
    }
}

pub struct VecWithDefault<T: Copy> {
    vec: Vec<T>,
    default: T,
}

impl<T: Copy> SearchArray<T> for VecWithDefault<T> {
    fn new(n: usize, default: T) -> Self {
        Self {
            vec: vec![default; n],
            default,
        }
    }

    fn get(&mut self, index: usize) -> T {
        self.vec[index]
    }

    fn set(&mut self, index: usize, value: T) {
        self.vec[index] = value;
    }

    fn reset(&mut self) {
        self.vec.fill(self.default);
    }
}

pub trait SearchArrayBackend {
    type Array<T: Copy>: SearchArray<T>;

    fn new<T: Copy>(n: usize, default: T) -> Self::Array<T> {
        Self::Array::<T>::new(n, default)
    }
}

pub struct FastClearArrayBackend;

impl SearchArrayBackend for FastClearArrayBackend {
    type Array<T: Copy> = FastClearArray<T>;
}

pub struct VecArrayBackend;

impl SearchArrayBackend for VecArrayBackend {
    type Array<T: Copy> = VecWithDefault<T>;
}

/// `Bfs` が探索する状態空間を定義する trait。
///
/// `S` は状態の型、`E` は辺の復元用情報の型。
/// `state_to_index` / `index_to_state` で状態と内部配列 index を相互変換し、
/// `next_states` で隣接状態を列挙する。
pub trait BfsContext<S: Copy, E: Copy = ()> {
    /// 状態 `s` を `0..max_state_size` の index に変換する。
    ///
    /// 返す index は `Bfs::new(max_state_size)` で指定した範囲内である必要がある。
    fn state_to_index(&self, s: S) -> usize;

    /// `state_to_index` で得た index から状態を復元する。
    fn index_to_state(&self, index: usize) -> S;

    /// `next_states` を呼ぶ直前に実行される hook。
    ///
    /// 隣接状態の列挙順をシャッフルするなど、展開ごとの前処理に使う。
    fn before_next_states(&mut self) {}

    /// 状態 `s` から遷移できる隣接状態を `buf` に追加する。
    ///
    /// `buf` は呼び出し前に空になっている。各要素は `(next_state, edge_info)`。
    /// `edge_info` は経路復元時に `Path::edges` へ格納される。
    fn next_states(&self, s: S, buf: &mut Vec<(S, E)>);
}

/// `Dijkstra` が探索する状態空間を定義する trait。
///
/// `S` は状態の型、`E` は辺の復元用情報の型。
/// `state_to_index` / `index_to_state` で状態と内部配列 index を相互変換し、
/// `next_states` で隣接状態と非負の遷移コストを列挙する。
pub trait DijkstraContext<S: Copy, E: Copy = ()> {
    /// 状態 `s` を `0..max_state_size` の index に変換する。
    ///
    /// 返す index は `Dijkstra::new(max_state_size)` で指定した範囲内である必要がある。
    fn state_to_index(&self, s: S) -> usize;

    /// `state_to_index` で得た index から状態を復元する。
    fn index_to_state(&self, index: usize) -> S;

    /// `next_states` を呼ぶ直前に実行される hook。
    ///
    /// 隣接状態の列挙順をシャッフルするなど、展開ごとの前処理に使う。
    fn before_next_states(&mut self) {}

    /// 状態 `s` から遷移できる隣接状態を `buf` に追加する。
    ///
    /// `buf` は呼び出し前に空になっている。各要素は `(next_state, cost, edge_info)`。
    /// `cost` は非負である必要がある。`edge_info` は経路復元時に `Path::edges` へ格納される。
    fn next_states(&self, s: S, buf: &mut Vec<(S, i64, E)>);
}

#[derive(Clone, Copy)]
struct Prev<E: Copy> {
    index: usize,
    edge: E,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path<S, E> {
    pub states: Vec<S>,
    pub edges: Vec<E>,
}

impl<S, E> Path<S, E> {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.states.clear();
        self.edges.clear();
    }
}

/// 汎用的な BFS 探索器。
///
/// `BfsContext` で状態空間を定義し、`find_path` に開始状態・終了条件・出力先を渡すと、
/// 終了条件を満たす状態までの最短経路を出力先に書き込む。
///
/// `E` は辺の復元用情報。`Path::edges[i]` は
/// `Path::states[i] -> Path::states[i + 1]` に対応する。
/// 辺情報が不要なら既定の `E = ()` を使う。
///
/// `A` は内部配列の実装。既定では `FastClearArrayBackend` を使う。
/// `VecArrayBackend` に差し替え可能。
///
/// # 例
///
/// ```ignore
/// use ahc_library::utils::bfs::{Bfs, BfsContext, Path};
///
/// struct Context;
///
/// impl BfsContext<usize> for Context {
///     fn state_to_index(&self, s: usize) -> usize { s }
///     fn index_to_state(&self, index: usize) -> usize { index }
///
///     fn next_states(&self, s: usize, buf: &mut Vec<(usize, ())>) {
///         if s + 1 < 10 {
///             buf.push((s + 1, ()));
///         }
///         if s + 2 < 10 {
///             buf.push((s + 2, ()));
///         }
///     }
/// }
///
/// let mut bfs = Bfs::new(10);
/// let mut ctx = Context;
/// let mut path = Path::new();
/// assert!(bfs.find_path(0, &mut ctx, |s| s == 9, &mut path));
/// assert_eq!(path.states.first(), Some(&0));
/// assert_eq!(path.states.last(), Some(&9));
/// ```
pub struct Bfs<E: Copy = (), A: SearchArrayBackend = VecArrayBackend> {
    dist: A::Array<i64>,
    prev: A::Array<Option<Prev<E>>>,
    q: VecDeque<usize>,
}

impl<E: Copy, A: SearchArrayBackend> Bfs<E, A> {
    pub fn new(max_state_size: usize) -> Self {
        Self {
            dist: A::new(max_state_size, INF_COST),
            prev: A::new(max_state_size, None),
            q: VecDeque::with_capacity(max_state_size),
        }
    }

    pub fn find_path<C, S, F>(
        &mut self,
        start: S,
        ctx: &mut C,
        complete_cond: F,
        buf: &mut Path<S, E>,
    ) -> bool
    where
        C: BfsContext<S, E>,
        S: Copy,
        F: FnMut(S) -> bool,
    {
        if let Some(end) = self.bfs(start, ctx, complete_cond) {
            self.restore_path(start, end, ctx, buf);
            true
        } else {
            buf.clear();
            false
        }
    }

    pub fn restore_path<C, S>(&mut self, start: S, end: S, ctx: &C, buf: &mut Path<S, E>)
    where
        C: BfsContext<S, E>,
        S: Copy,
    {
        buf.clear();
        let start_index = ctx.state_to_index(start);
        let end_index = ctx.state_to_index(end);
        buf.states.push(ctx.index_to_state(end_index));
        let mut cur = end_index;
        while let Some(prev) = self.prev.get(cur) {
            cur = prev.index;
            buf.states.push(ctx.index_to_state(cur));
            buf.edges.push(prev.edge);
        }
        buf.states.reverse();
        buf.edges.reverse();
        assert_eq!(cur, start_index);
    }

    /// `start` から到達可能な全状態への BFS 距離を計算する。
    ///
    /// 計算後、各状態の距離は `get_dist` で取得できる。
    pub fn calc_dist<C, S>(&mut self, start: S, ctx: &mut C)
    where
        C: BfsContext<S, E>,
        S: Copy,
    {
        self.bfs(start, ctx, |_| false);
    }

    /// 最後に実行した探索における `s` への距離を返す。
    ///
    /// 未到達なら `None` を返す。`find_path` は終了条件を満たした時点で探索を止めるため、
    /// 全状態への距離が必要な場合は先に `calc_dist` を実行する。
    pub fn get_dist<C, S>(&mut self, s: S, ctx: &C) -> Option<i64>
    where
        C: BfsContext<S, E>,
        S: Copy,
    {
        let dist = self.dist.get(ctx.state_to_index(s));
        (dist != INF_COST).then_some(dist)
    }

    pub fn bfs<C, S, F>(&mut self, start: S, ctx: &mut C, mut complete_cond: F) -> Option<S>
    where
        C: BfsContext<S, E>,
        S: Copy,
        F: FnMut(S) -> bool,
    {
        self.reset();

        let start_index = ctx.state_to_index(start);
        self.dist.set(start_index, 0);
        self.q.push_back(start_index);

        let mut next_states = Vec::new();

        while let Some(index) = self.q.pop_front() {
            let s = ctx.index_to_state(index);
            if complete_cond(s) {
                return Some(s);
            }
            let new_dist = self.dist.get(index) + 1;
            ctx.before_next_states();
            next_states.clear();
            ctx.next_states(s, &mut next_states);
            for (next, edge) in next_states.drain(..) {
                let next_index = ctx.state_to_index(next);
                if self.dist.get(next_index) <= new_dist {
                    continue;
                }
                self.dist.set(next_index, new_dist);
                self.prev.set(next_index, Some(Prev { index, edge }));
                self.q.push_back(next_index);
            }
        }

        None
    }

    fn reset(&mut self) {
        self.q.clear();
        self.dist.reset();
        self.prev.reset();
    }
}

/// 汎用的な Dijkstra 探索器。
///
/// `DijkstraContext` で状態空間を定義し、`find_path` に開始状態・終了条件・出力先を渡すと、
/// 終了条件を満たす状態までの最小コスト経路を出力先に書き込む。
///
/// `E` は辺の復元用情報。`Path::edges[i]` は
/// `Path::states[i] -> Path::states[i + 1]` に対応する。
/// 辺情報が不要なら既定の `E = ()` を使う。
///
/// `A` は内部配列の実装。既定では `FastClearArrayBackend` を使う。
/// `VecArrayBackend` に差し替え可能。
///
/// # 例
///
/// ```ignore
/// use ahc_library::utils::bfs::{Dijkstra, DijkstraContext, Path};
///
/// struct Context;
///
/// impl DijkstraContext<usize> for Context {
///     fn state_to_index(&self, s: usize) -> usize { s }
///     fn index_to_state(&self, index: usize) -> usize { index }
///
///     fn next_states(&self, s: usize, buf: &mut Vec<(usize, i64, ())>) {
///         if s + 1 < 10 {
///             buf.push((s + 1, 1, ()));
///         }
///         if s + 2 < 10 {
///             buf.push((s + 2, 3, ()));
///         }
///     }
/// }
///
/// let mut dijkstra = Dijkstra::new(10);
/// let mut ctx = Context;
/// let mut path = Path::new();
/// assert!(dijkstra.find_path(0, &mut ctx, |s| s == 9, &mut path));
/// assert_eq!(path.states.first(), Some(&0));
/// assert_eq!(path.states.last(), Some(&9));
/// assert_eq!(dijkstra.get_dist(9, &ctx), Some(9));
/// ```
pub struct Dijkstra<E: Copy = (), A: SearchArrayBackend = VecArrayBackend> {
    dist: A::Array<i64>,
    prev: A::Array<Option<Prev<E>>>,
    heap: BinaryHeap<Reverse<(i64, usize)>>,
}

impl<E: Copy, A: SearchArrayBackend> Dijkstra<E, A> {
    pub fn new(max_state_size: usize) -> Self {
        Self {
            dist: A::new(max_state_size, INF_COST),
            prev: A::new(max_state_size, None),
            heap: BinaryHeap::with_capacity(max_state_size),
        }
    }

    pub fn find_path<C, S, F>(
        &mut self,
        start: S,
        ctx: &mut C,
        complete_cond: F,
        buf: &mut Path<S, E>,
    ) -> bool
    where
        C: DijkstraContext<S, E>,
        S: Copy,
        F: FnMut(S) -> bool,
    {
        if let Some(end) = self.dijkstra(start, ctx, complete_cond) {
            self.restore_path(start, end, ctx, buf);
            true
        } else {
            buf.clear();
            false
        }
    }

    pub fn restore_path<C, S>(&mut self, start: S, end: S, ctx: &C, buf: &mut Path<S, E>)
    where
        C: DijkstraContext<S, E>,
        S: Copy,
    {
        buf.clear();
        let start_index = ctx.state_to_index(start);
        let end_index = ctx.state_to_index(end);
        buf.states.push(ctx.index_to_state(end_index));
        let mut cur = end_index;
        while let Some(prev) = self.prev.get(cur) {
            cur = prev.index;
            buf.states.push(ctx.index_to_state(cur));
            buf.edges.push(prev.edge);
        }
        buf.states.reverse();
        buf.edges.reverse();
        assert_eq!(cur, start_index);
    }

    /// `start` から到達可能な全状態への最小コストを計算する。
    ///
    /// 計算後、各状態の距離は `get_dist` で取得できる。
    pub fn calc_dist<C, S>(&mut self, start: S, ctx: &mut C)
    where
        C: DijkstraContext<S, E>,
        S: Copy,
    {
        self.dijkstra(start, ctx, |_| false);
    }

    /// 最後に実行した探索における `s` への最小コストを返す。
    ///
    /// 未到達なら `None` を返す。`find_path` は終了条件を満たした時点で探索を止めるため、
    /// 全状態への距離が必要な場合は先に `calc_dist` を実行する。
    pub fn get_dist<C, S>(&mut self, s: S, ctx: &C) -> Option<i64>
    where
        C: DijkstraContext<S, E>,
        S: Copy,
    {
        let dist = self.dist.get(ctx.state_to_index(s));
        (dist != INF_COST).then_some(dist)
    }

    pub fn dijkstra<C, S, F>(&mut self, start: S, ctx: &mut C, mut complete_cond: F) -> Option<S>
    where
        C: DijkstraContext<S, E>,
        S: Copy,
        F: FnMut(S) -> bool,
    {
        self.reset();

        let start_index = ctx.state_to_index(start);
        self.dist.set(start_index, 0);
        self.heap.push(Reverse((0, start_index)));

        let mut next_states = Vec::new();

        while let Some(Reverse((cur_dist, index))) = self.heap.pop() {
            if self.dist.get(index) != cur_dist {
                continue;
            }

            let s = ctx.index_to_state(index);
            if complete_cond(s) {
                return Some(s);
            }

            ctx.before_next_states();
            next_states.clear();
            ctx.next_states(s, &mut next_states);
            for (next, cost, edge) in next_states.drain(..) {
                debug_assert!(cost >= 0, "Dijkstra requires non-negative edge costs");
                let next_dist = cur_dist + cost;
                let next_index = ctx.state_to_index(next);
                if self.dist.get(next_index) <= next_dist {
                    continue;
                }
                self.dist.set(next_index, next_dist);
                self.prev.set(next_index, Some(Prev { index, edge }));
                self.heap.push(Reverse((next_dist, next_index)));
            }
        }

        None
    }

    fn reset(&mut self) {
        self.heap.clear();
        self.dist.reset();
        self.prev.reset();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::utils::{
        random::{Random, XorShift32},
        v2::{Coor, D4},
    };

    use super::*;

    const H: usize = 3;
    const W: usize = 3;
    type C = Coor<usize>;

    struct RandomGridContext {
        d4: [C; 4],
        rng: XorShift32,
    }

    impl RandomGridContext {
        fn new(seed: u32) -> Self {
            Self {
                d4: D4,
                rng: XorShift32::new(seed),
            }
        }
    }

    impl BfsContext<C, C> for RandomGridContext {
        fn state_to_index(&self, s: C) -> usize {
            s.i * W + s.j
        }

        fn index_to_state(&self, index: usize) -> C {
            C::new(index / W, index % W)
        }

        fn before_next_states(&mut self) {
            self.rng.shuffle(&mut self.d4);
        }

        fn next_states(&self, s: C, buf: &mut Vec<(C, C)>) {
            for &d in &self.d4 {
                let next = s + d;
                if next.i < H && next.j < W {
                    buf.push((next, d));
                }
            }
        }
    }

    fn assert_bfs_find_path_with_randomized_next_states<A: SearchArrayBackend>() {
        let mut bfs = Bfs::<C, A>::new(H * W);
        let mut ctx = RandomGridContext::new(42);
        let start = C::new(0, 0);
        let goal = C::new(H - 1, W - 1);
        let mut paths = BTreeSet::new();

        let mut path = Path::new();
        for _ in 0..50 {
            assert!(bfs.find_path(start, &mut ctx, |s| s == goal, &mut path));
            assert_eq!(path.states.first(), Some(&start));
            assert_eq!(path.states.last(), Some(&goal));
            assert_eq!(path.states.len(), H + W - 1);
            assert_eq!(path.edges.len() + 1, path.states.len());
            for i in 0..path.edges.len() {
                assert_eq!(path.states[i] + path.edges[i], path.states[i + 1]);
            }
            paths.insert(path.states.clone());
        }

        assert!(
            paths.len() >= 2,
            "expected randomized next_states to produce multiple shortest paths, got {paths:?}"
        );
    }

    #[test]
    fn test_bfs_find_path_with_randomized_next_states() {
        assert_bfs_find_path_with_randomized_next_states::<FastClearArrayBackend>();
    }

    #[test]
    fn test_bfs_find_path_with_vec_array() {
        assert_bfs_find_path_with_randomized_next_states::<VecArrayBackend>();
    }

    struct BlockedGridContext;

    impl BfsContext<C> for BlockedGridContext {
        fn state_to_index(&self, s: C) -> usize {
            s.i * W + s.j
        }

        fn index_to_state(&self, index: usize) -> C {
            C::new(index / W, index % W)
        }

        fn next_states(&self, s: C, buf: &mut Vec<(C, ())>) {
            for &d in &D4 {
                let next = s + d;
                if next.i < H && next.j < W && next != C::new(1, 1) {
                    buf.push((next, ()));
                }
            }
        }
    }

    #[test]
    fn test_bfs_calc_dist() {
        let mut bfs = Bfs::<()>::new(H * W);
        let mut ctx = BlockedGridContext;

        bfs.calc_dist(C::new(0, 0), &mut ctx);

        assert_eq!(bfs.get_dist(C::new(0, 0), &ctx), Some(0));
        assert_eq!(bfs.get_dist(C::new(0, 1), &ctx), Some(1));
        assert_eq!(bfs.get_dist(C::new(1, 1), &ctx), None);
        assert_eq!(bfs.get_dist(C::new(2, 2), &ctx), Some(4));
    }

    struct WeightedGraphContext;

    impl DijkstraContext<usize, i64> for WeightedGraphContext {
        fn state_to_index(&self, s: usize) -> usize {
            s
        }

        fn index_to_state(&self, index: usize) -> usize {
            index
        }

        fn next_states(&self, s: usize, buf: &mut Vec<(usize, i64, i64)>) {
            match s {
                0 => {
                    buf.push((1, 100, 100));
                    buf.push((2, 1, 1));
                }
                1 => {
                    buf.push((3, 1, 1));
                }
                2 => {
                    buf.push((1, 1, 1));
                    buf.push((3, 100, 100));
                }
                _ => {}
            }
        }
    }

    fn assert_dijkstra_find_path<A: SearchArrayBackend>() {
        let mut dijkstra = Dijkstra::<i64, A>::new(5);
        let mut ctx = WeightedGraphContext;
        let mut path = Path::new();

        assert!(dijkstra.find_path(0, &mut ctx, |s| s == 3, &mut path));
        assert_eq!(path.states, vec![0, 2, 1, 3]);
        assert_eq!(path.edges, vec![1, 1, 1]);
        assert_eq!(dijkstra.get_dist(3, &ctx), Some(3));
    }

    #[test]
    fn test_dijkstra_find_path() {
        assert_dijkstra_find_path::<FastClearArrayBackend>();
    }

    #[test]
    fn test_dijkstra_find_path_with_vec_array() {
        assert_dijkstra_find_path::<VecArrayBackend>();
    }

    #[test]
    fn test_dijkstra_calc_dist() {
        let mut dijkstra = Dijkstra::<i64>::new(5);
        let mut ctx = WeightedGraphContext;

        dijkstra.calc_dist(0, &mut ctx);

        assert_eq!(dijkstra.get_dist(0, &ctx), Some(0));
        assert_eq!(dijkstra.get_dist(1, &ctx), Some(2));
        assert_eq!(dijkstra.get_dist(2, &ctx), Some(1));
        assert_eq!(dijkstra.get_dist(3, &ctx), Some(3));
        assert_eq!(dijkstra.get_dist(4, &ctx), None);
    }
}
