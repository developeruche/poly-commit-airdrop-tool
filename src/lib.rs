use std::fs::File;
use std::io::{Write, BufReader, BufRead, Error};



pub fn write_to_file(path: &str, data: &str) {
    let mut file = File::create(path).unwrap();
    file.write_all(data.as_bytes()).unwrap();
}