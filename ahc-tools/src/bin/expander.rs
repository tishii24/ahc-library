use ahc_tools::expander::core::{bundle_solution, BundleRequest};
use clap::Parser;
use std::{env, path::PathBuf};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long = "ahc_library_path")]
    ahc_library_path: Option<PathBuf>,
    #[arg(long = "solution_path", default_value = ".")]
    solution_path: PathBuf,
    #[arg(long = "skip_rustfmt", default_value_t = false)]
    skip_rustfmt: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let ahc_library_path = args
        .ahc_library_path
        .or_else(|| env::var_os("AHC_LIBRARY_PATH").map(PathBuf::from));

    let bundled = bundle_solution(&BundleRequest {
        solution_path: args.solution_path,
        ahc_library_path,
        rustfmt: !args.skip_rustfmt,
    })?;

    println!("{}", bundled);
    Ok(())
}
