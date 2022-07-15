#![allow(unused_mut)]
#![allow(unused_imports)]

use super::dynamic_vector::DynamicBitVector;
use std::fs::{write, File, OpenOptions};
use std::io::stdin;
use std::io::{self, BufRead, Write};
use std::path::Path;
use crate::traits::{StaticBitVec, DynBitVec};

/// Read large files line by line in Rust
/// Efficient (cache) implementations to read file line-by-line
/// <https://stackoverflow.com/questions/45882329/read-large-files-line-by-line-in-rust>
///
/// The output is wrapped in a Result to allow matching on errors
/// Returns an Iterator to the Reader of the lines of the file.
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn insert(mut vec: DynamicBitVector, command: Vec<&str>) -> DynamicBitVector {
    let index = command[1].parse::<usize>().unwrap();
    // let bit = command[2].parse::<bool>().unwrap();
    let bit = command[2] != "0";
    vec.insert(index, bit);
    vec
}

pub fn delete(mut vec: DynamicBitVector, command: Vec<&str>) -> DynamicBitVector {
    let index = command[1].parse::<usize>().unwrap();
    vec.delete(index);
    vec
}

pub fn flip(mut vec: DynamicBitVector, command: Vec<&str>) -> DynamicBitVector {
    let index = command[1].parse::<usize>().unwrap();
    vec.flip(index);
    vec
}

pub fn rank(mut vec: DynamicBitVector, command: Vec<&str>) -> (usize, DynamicBitVector) {
    let bit = command[1] != "0";
    let index = command[2].parse::<usize>().unwrap();
    let rank = vec.rank(bit, index);
    (rank, vec)
}

pub fn select(mut vec: DynamicBitVector, command: Vec<&str>) -> (usize, DynamicBitVector) {
    let bit = command[1] != "0";
    let index = command[2].parse::<usize>().unwrap();
    let sel = vec.select(bit, index);
    (sel, vec)
}

/// Write `text` to (non-) existing `filename`, overwriting it.
pub fn write_file<P>(filename: P, text: &str) -> std::io::Result<()>
where
    P: AsRef<Path>,
{
    write(filename, text)?;
    Ok(())
}

/// Appending `text` to existing file with `filename` after newline. Creates file if it does not
/// exist yet.
pub fn append_file<P>(filename: P, val: usize) -> std::io::Result<()>
where
    P: AsRef<Path>,
{
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();

    // file.write_all(b"to append");
    // or
    write!(file, "\n{}", val)
}

pub fn wait_continue() {
    let mut input_string = String::new();
    stdin().read_line(&mut input_string).ok().unwrap();
}
