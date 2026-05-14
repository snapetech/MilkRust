use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let code = milkrust_desktop::run_desktop_probe(&args);
    process::exit(code);
}
