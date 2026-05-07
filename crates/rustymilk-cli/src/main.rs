use std::{env, process};

use rustymilk_cli::run_rustymilk_cli;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = run_rustymilk_cli(&args);
    if !result.stdout.is_empty() {
        print!("{}", result.stdout);
    }
    if !result.stderr.is_empty() {
        eprint!("{}", result.stderr);
    }
    process::exit(result.code);
}
