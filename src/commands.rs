use super::avl_tree::AVL;
use super::dynamic_vector::DynamicBitVector;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

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

pub fn insert(mut tree: AVL, command: Vec<&str>) -> AVL {
    let index = command[1].parse::<usize>().unwrap();
    // let bit = command[2].parse::<bool>().unwrap();
    let bit = command[2] != "0";
    tree.insert(index, bit);
    return tree;
}

pub fn delete(ref mut vec: DynamicBitVector, command: Vec<&str>) {
    let index = command[1].parse::<usize>().unwrap();
    vec.delete(index);
}


pub fn flip(ref mut vec: DynamicBitVector, command: Vec<&str>) {
    let index = command[1].parse::<usize>().unwrap();
    vec.flip(index);
}

pub fn rank(ref mut vec: DynamicBitVector, command: Vec<&str>) {
    let bit = command[1] != "0";
    let index = command[2].parse::<usize>().unwrap();
    vec.rank(bit, index);
}

pub fn select(ref mut vec: DynamicBitVector, command: Vec<&str>) {
    let bit = command[1] != "0";
    let index = command[2].parse::<usize>().unwrap();
    vec.select(bit, index);
}
