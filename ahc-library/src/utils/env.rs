pub const fn env_is_one(opt: Option<&str>) -> bool {
    match opt {
        Some(s) => {
            let b = s.as_bytes();
            b.len() == 1 && b[0] == b'1'
        }
        None => false,
    }
}
