use std::fs;

fn untrusted_len_to_capacity(input: &str) -> Vec<u8> {
    let len: usize = input.parse().unwrap();
    Vec::with_capacity(len)
}

fn untrusted_path_to_file(path: &str) -> String {
    fs::read_to_string(path).unwrap()
}
