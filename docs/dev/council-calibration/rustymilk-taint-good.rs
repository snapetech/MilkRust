use std::fs;
use std::path::{Path, PathBuf};

fn bounded_len_to_capacity(input: &str) -> Vec<u8> {
    let len: usize = input.parse().unwrap();
    let capped = len.min(4096);
    Vec::with_capacity(capped)
}

fn contained_path_to_file(root: &Path, relative: &str) -> String {
    let path: PathBuf = root.join(relative);
    let canonical = path.canonicalize().unwrap();
    assert!(canonical.starts_with(root));
    fs::read_to_string(canonical).unwrap()
}
