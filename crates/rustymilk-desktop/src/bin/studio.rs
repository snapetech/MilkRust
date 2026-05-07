use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let code = rustymilk_desktop::run_desktop_studio(&args);
    process::exit(code);
}
