use ahc_library::params_impl;
use proconio::input;

params_impl! {
    { n_coef: f64, m_coef: f64 },
    [
        "0" => { n_coef: 5.2138591882429415, m_coef: 3.9377773177132744 },
        "1" => { n_coef: 5.1331404919100825, m_coef: 3.5523832780594145 },
        _ => { n_coef: 5.000000, m_coef: 5.000000 },
    ]
}

fn group_id_fn(n: usize, m: usize) -> String {
    if n < 100 && m < 100 {
        "0".to_string()
    } else {
        "1".to_string()
    }
}
// params_impl! {
//     n_coef: f64 = 5.0,
//     m_coef: f64 = 5.0,
// }

fn main() {
    input! {
        n: usize,
        m: usize,
    }
    let group_id = group_id_fn(n, m);
    let params = Params::load(&group_id);

    let x = n as i64 * params.n_coef as i64 + m as i64 * params.m_coef as i64;
    println!("{}", x);
}
