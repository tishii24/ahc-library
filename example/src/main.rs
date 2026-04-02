mod helper;

use ahc_library::dumpln;
use ahc_library::params_impl;
use proconio::input;

use crate::helper::f;

params_impl! {
    n_coef: f64 = 3.9572835556899086,
    m_coef: f64 = 7.0385499979441235,
}

// fn group_id_fn(n: usize, m: usize) -> String {
//     if n < 50 && m < 50 {
//         "0".to_string()
//     } else if n < 101 && m < 50 {
//         "1".to_string()
//     } else if n < 50 && m < 101 {
//         "2".to_string()
//     } else {
//         "3".to_string()
//     }
// }

// params_impl! {
//     { n_coef: f64, m_coef: f64 },
//     [
//         "0" => { n_coef: 5.911064041309099, m_coef: 3.404873337327698 },
//         "1" => { n_coef: 2.574560011770543, m_coef: 5.910610920190062 },
//         "2" => { n_coef: 3.5737121598862833, m_coef: 7.571256449897406 },
//         "3" => { n_coef: 4.728672779566716, m_coef: 8.0125115098411 },
//         _ => { n_coef: 5.0, m_coef: 5.0 },
//     ]
// }

fn main() {
    input! {
        n: usize,
        m: usize,
    }
    let params = Params::load();
    // let group_id = group_id_fn(n, m);
    // let params = Params::load(&group_id);

    let x = n as i64 * params.n_coef as i64 + m as i64 * params.m_coef as i64;

    println!("{}", x);

    dumpln!("hi");

    f();
}
