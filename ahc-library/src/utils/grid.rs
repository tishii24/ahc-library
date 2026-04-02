const N: usize = 1 << 9;

/// 3×3 マス目上の関節点（除去すると連結でなくなる頂点）を高速に判定する構造体。
///
/// 3×3 グリッドの部分集合を 9 ビットのビットマスクで表し、
/// 事前に全 512 通りの連結性を計算したルックアップテーブルを用いる。
/// これにより関節点の判定を O(1) で行える。
///
/// # ビットマスクの形式
///
/// インデックス `i` のビット（`1 << i`）は、3×3 グリッドを左上から
/// 行優先で並べたときの位置 `i`（0〜8）のマスが存在することを表す：
///
/// ```text
/// 0 1 2
/// 3 4 5
/// 6 7 8
/// ```
#[derive(Debug, Clone)]
pub struct EasyArticulationChecker {
    connected: [bool; N],
}

impl Default for EasyArticulationChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl EasyArticulationChecker {
    pub fn new() -> Self {
        Self {
            connected: Self::make_connected(),
        }
    }

    /// 中心マスを追加した時に連結になるか判定する
    /// `grid`は中心マス周辺の8マスのビットマスク（0〜255）で与える
    /// 中心マスに対応するマス（=4bit目）は0である必要がある
    pub fn is_articulation(&self, grid: usize) -> bool {
        debug_assert!(
            grid & (1 << 4) == 0,
            "{:09b} is not a valid grid mask",
            grid
        );
        self.connected[grid | (1 << 4)] && !self.connected[grid]
    }

    fn make_connected() -> [bool; N] {
        let mut edges = vec![Vec::new(); 9];
        for v in 0..9 {
            if v / 3 > 0 {
                edges[v].push(v - 3);
            }
            if v % 3 > 0 {
                edges[v].push(v - 1);
            }
            if v % 3 < 2 {
                edges[v].push(v + 1);
            }
            if v / 3 < 2 {
                edges[v].push(v + 3);
            }
        }
        let mut connected = [false; N];
        connected[0] = true;
        for s in 1..N {
            let root = s.trailing_zeros();
            let mut visited = 1 << root;
            let mut todo: usize = 1 << root;
            while todo > 0 {
                let u = todo.trailing_zeros() as usize;
                todo ^= 1 << u;
                for &v in &edges[u] {
                    if (s & (1 << v)) > 0 && (visited & (1 << v)) == 0 {
                        visited |= 1 << v;
                        todo |= 1 << v;
                    }
                }
            }
            if visited == s {
                connected[s] = true;
            }
        }
        connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_mask(a: [[usize; 3]; 3]) -> usize {
        let mut mask = 0;
        for i in 0..3 {
            for j in 0..3 {
                if a[i][j] > 0 {
                    mask |= 1 << (i * 3 + j);
                }
            }
        }
        eprintln!("mask: {:09b}", mask);
        mask
    }

    #[test]
    fn test_articulation() {
        let checker = EasyArticulationChecker::new();

        let a = [[0, 0, 0], [0, 0, 0], [0, 0, 0]];
        assert!(!checker.is_articulation(to_mask(a)));

        let a = [[0, 1, 0], [0, 0, 0], [0, 1, 0]];
        assert!(checker.is_articulation(to_mask(a)));

        let a = [[0, 0, 0], [1, 0, 1], [0, 0, 0]];
        assert!(checker.is_articulation(to_mask(a)));

        let a = [[1, 1, 1], [0, 0, 0], [0, 0, 0]];
        assert!(!checker.is_articulation(to_mask(a)));

        let a = [[1, 1, 1], [0, 0, 0], [1, 1, 1]];
        assert!(checker.is_articulation(to_mask(a)));

        let a = [[1, 0, 1], [0, 0, 0], [1, 0, 1]];
        assert!(!checker.is_articulation(to_mask(a)));

        let a = [[1, 1, 0], [1, 0, 0], [1, 1, 0]];
        assert!(!checker.is_articulation(to_mask(a)));

        let a = [[1, 1, 0], [1, 0, 1], [0, 1, 1]];
        assert!(checker.is_articulation(to_mask(a)));
    }
}
