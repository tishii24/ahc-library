static mut START: f64 = -1.;

#[allow(unused)]
pub fn start_clock() {
    let _ = elapsed_seconds();
}

#[inline]
#[allow(unused)]
pub fn elapsed_seconds() -> f64 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    unsafe {
        if START < 0. {
            START = t;
        }
        t - START
    }
}
