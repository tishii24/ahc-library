#![allow(non_snake_case, dead_code, unused_imports, unused_macros)]

use proconio::{input, marker::*, source::Source};
use rand::prelude::*;
use std::io::prelude::*;
use svg::node::element::{path::Data, Circle, Path, Rectangle};

pub type Output = i64;

pub struct Input {
    pub N: usize,
    pub M: usize,
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", self.N, self.M)?;
        Ok(())
    }
}

pub fn parse_input(f: &str) -> Input {
    let f = proconio::source::once::OnceSource::from(f);
    input! {
        from f,
        N: usize,
        M: usize,
    }
    Input { N, M }
}

pub fn parse_output(_input: &Input, f: &str) -> Output {
    let f = proconio::source::once::OnceSource::from(f);
    input! {
        from f,
        s: i64,
    }
    s
}

pub fn compute_score_detail(input: &Input, out: &Output) -> (i64, String) {
    let ans = if input.N < 50 && input.M < 50 {
        (input.N * 5 + input.M * 3) as i64
    } else if input.N < 101 && input.M < 50 {
        (input.N * 2 + input.M * 5) as i64
    } else if input.N < 50 && input.M < 101 {
        (input.N * 3 + input.M * 7) as i64
    } else {
        (input.N * 4 + input.M * 8) as i64
    };
    let score = (ans - out).pow(2) + 1;
    (score, String::new())
}

pub fn gen_input(seed: u64) -> Input {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed ^ 10);
    let N = (rng.gen_range(1..=100)) as usize;
    let M = (rng.gen_range(1..=100)) as usize;
    Input { N, M }
}

pub fn vis_default(input: &Input, out: &Output) -> (i64, String, String) {
    let (score, svg, err) = vis(input, out, true, i64::max_value());
    (score, svg, err)
}

pub fn vis(input: &Input, out: &Output, _show_number: bool, _t: i64) -> (i64, String, String) {
    let (score, err) = compute_score_detail(input, out);
    (score, "".to_string(), err)
}
