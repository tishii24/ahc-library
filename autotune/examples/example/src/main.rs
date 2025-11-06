use ahc_library::params_impl;
use proconio::input;

// params_impl! {
//     { n: usize, m: usize },
//     { n_coef: f64, m_coef: f64 },
//     [
//         _ => { n_coef: 10.00000, m_coef: 10.00000 },
//     ]
// }
params_impl! {
    n_coef: f64 = 5.0,
    m_coef: f64 = 5.0,
}

fn main() {
    input! {
        n: usize,
        m: usize,
    }

    let params = Params::load();

    let x = n as i64 * params.n_coef as i64 + m as i64 * params.m_coef as i64;
    println!("{}", x);
}
