use ahc_library::dumpln;
use ahc_library::perf;

pub fn f() {
    perf!("hi");
    dumpln!("hello");
}
