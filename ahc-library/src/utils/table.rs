use crate::utils::fast_clear_array::FastClearArray;

pub const INF_COST: i64 = i64::MAX / 2;

pub trait TableArray<T: Copy> {
    fn new(n: usize, default: T) -> Self;
    fn get(&mut self, index: usize) -> T;
    fn set(&mut self, index: usize, value: T);
    fn reset(&mut self);
}

impl<T: Copy> TableArray<T> for FastClearArray<T> {
    fn new(n: usize, default: T) -> Self {
        FastClearArray::new(n, default)
    }

    #[inline]
    fn get(&mut self, index: usize) -> T {
        FastClearArray::get(self, index)
    }

    #[inline]
    fn set(&mut self, index: usize, value: T) {
        FastClearArray::set(self, index, value);
    }

    #[inline]
    fn reset(&mut self) {
        self.clear();
    }
}

pub struct VecTableArray<T: Copy> {
    values: Vec<T>,
    default: T,
}

impl<T: Copy> TableArray<T> for VecTableArray<T> {
    fn new(n: usize, default: T) -> Self {
        Self {
            values: vec![default; n],
            default,
        }
    }

    #[inline]
    fn get(&mut self, index: usize) -> T {
        self.values[index]
    }

    #[inline]
    fn set(&mut self, index: usize, value: T) {
        self.values[index] = value;
    }

    #[inline]
    fn reset(&mut self) {
        self.values.fill(self.default);
    }
}

pub trait TableArrayBackend {
    type Array<T: Copy>: TableArray<T>;

    fn new<T: Copy>(n: usize, default: T) -> Self::Array<T> {
        Self::Array::<T>::new(n, default)
    }
}

pub struct VecTableBackend;

impl TableArrayBackend for VecTableBackend {
    type Array<T: Copy> = VecTableArray<T>;
}

pub struct FastClearTableBackend;

impl TableArrayBackend for FastClearTableBackend {
    type Array<T: Copy> = FastClearArray<T>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PrevIndex<E: Copy> {
    index: usize,
    edge: E,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Prev<S: Copy, E: Copy> {
    pub state: S,
    pub edge: E,
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

impl<S, E> Default for Path<S, E> {
    fn default() -> Self {
        Self::new()
    }
}

/// 探索中の状態ごとの距離と直前状態を管理する dense table。
///
/// 内部では状態 `S` を `0..len` の index に変換して保持する。
/// 公開 API は基本的に `S` を受け取り、`S` を返す。
///
/// `prev` は経路復元用の情報で、`Path::edges[i]` は
/// `Path::states[i] -> Path::states[i + 1]` に対応する。
pub struct SearchTable<S, SI, IS, E = (), A = VecTableBackend>
where
    S: Copy,
    SI: Fn(S) -> usize,
    IS: Fn(usize) -> S,
    E: Copy,
    A: TableArrayBackend,
{
    len: usize,
    to_index: SI,
    to_state: IS,
    dist: A::Array<i64>,
    prev: A::Array<Option<PrevIndex<E>>>,
}

impl<S, ToIndex, ToState, E, A> SearchTable<S, ToIndex, ToState, E, A>
where
    S: Copy,
    ToIndex: Fn(S) -> usize,
    ToState: Fn(usize) -> S,
    E: Copy,
    A: TableArrayBackend,
{
    pub fn new(len: usize, to_index: ToIndex, to_state: ToState) -> Self {
        Self {
            len,
            to_index,
            to_state,
            dist: A::new(len, INF_COST),
            prev: A::new(len, None),
        }
    }

    /// 全状態を未到達に戻す。
    pub fn reset(&mut self) {
        self.dist.reset();
        self.prev.reset();
    }

    /// `state` を開始状態として登録する。
    ///
    /// 距離は `0`、直前状態は `None` になる。
    /// 複数始点の場合は各始点に対してこのメソッドを呼ぶ。
    #[inline]
    pub fn set_start(&mut self, state: S) {
        let index = self.index_of(state);
        debug_assert!(index < self.len);
        self.dist.set(index, 0);
        self.prev.set(index, None);
    }

    /// `state` への距離を返す。未到達なら `None` を返す。
    #[inline]
    pub fn dist(&mut self, state: S) -> Option<i64> {
        let index = self.index_of(state);
        let dist = self.dist.get(index);
        (dist != INF_COST).then_some(dist)
    }

    #[inline]
    pub fn is_reached(&mut self, state: S) -> bool {
        self.dist(state).is_some()
    }

    /// `to` への距離が改善するなら、距離と直前状態を更新する。
    ///
    /// 更新した場合は `true`、既存距離以下で改善しない場合は `false` を返す。
    #[inline]
    pub fn try_update(&mut self, to: S, new_dist: i64, from: S, edge: E) -> bool {
        let to_index = self.index_of(to);
        let from_index = self.index_of(from);
        debug_assert!(new_dist < INF_COST);
        debug_assert!(
            self.dist.get(from_index) != INF_COST,
            "previous state is not reached"
        );

        if self.dist.get(to_index) <= new_dist {
            return false;
        }

        self.dist.set(to_index, new_dist);
        self.prev.set(
            to_index,
            Some(PrevIndex {
                index: from_index,
                edge,
            }),
        );
        true
    }

    /// `end` から `prev` を辿り、開始状態までの経路を復元する。
    ///
    /// 複数始点の場合、実際に到達元になった始点が `Path::states[0]` になる。
    /// `end` が未到達なら `buf` を空にして `false` を返す。
    pub fn restore_path(&mut self, end: S, buf: &mut Path<S, E>) -> bool {
        let end_index = self.index_of(end);
        buf.clear();

        if self.dist.get(end_index) == INF_COST {
            return false;
        }

        let mut cur = end_index;
        buf.states.push(self.state_of(cur));

        while let Some(prev) = self.prev.get(cur) {
            cur = prev.index;
            buf.states.push(self.state_of(cur));
            buf.edges.push(prev.edge);
        }

        buf.states.reverse();
        buf.edges.reverse();
        true
    }

    #[inline]
    fn index_of(&self, state: S) -> usize {
        let index = (self.to_index)(state);
        debug_assert!(
            index < self.len,
            "state index out of bounds: index={index}, len={}",
            self.len
        );
        index
    }

    #[inline]
    fn state_of(&self, index: usize) -> S {
        debug_assert!(
            index < self.len,
            "state index out of bounds: index={index}, len={}",
            self.len
        );
        (self.to_state)(index)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[test]
    fn test_search_table_with_grid_bfs() {
        const H: usize = 3;
        const W: usize = 4;

        let start = (0usize, 0usize);
        let goal = (2usize, 3usize);
        let blocked = (1usize, 1usize);
        let directions = [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)];

        let mut table = SearchTable::<_, _, _, (isize, isize), VecTableBackend>::new(
            H * W,
            |s: (usize, usize)| s.0 * W + s.1,
            |index| (index / W, index % W),
        );
        let mut q = VecDeque::new();

        table.set_start(start);
        q.push_back(start);

        while let Some(cur) = q.pop_front() {
            let next_dist = table.dist(cur).unwrap() + 1;

            for &edge @ (di, dj) in &directions {
                let ni = cur.0 as isize + di;
                let nj = cur.1 as isize + dj;
                if ni < 0 || H as isize <= ni || nj < 0 || W as isize <= nj {
                    continue;
                }

                let next = (ni as usize, nj as usize);
                if next == blocked {
                    continue;
                }

                if table.try_update(next, next_dist, cur, edge) {
                    q.push_back(next);
                }
            }
        }

        assert_eq!(table.dist(goal), Some(5));
        assert_eq!(table.dist(blocked), None);
        assert!(table.is_reached(goal));

        let mut path = Path::new();
        assert!(table.restore_path(goal, &mut path));
        assert_eq!(
            path.states,
            vec![(0, 0), (1, 0), (2, 0), (2, 1), (2, 2), (2, 3)]
        );
        assert_eq!(path.edges, vec![(1, 0), (1, 0), (0, 1), (0, 1), (0, 1)]);
    }
}
