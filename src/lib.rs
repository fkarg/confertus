#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

/// Module for parsing and building cli commands and args
pub mod commands;

/// Static bit vector implementation: `SBitVec` used as Leaf for dynamic bit vectors `DynBitV`
pub mod static_vector;

/// Dynamic Bit vector implementation for `Vec`: `V`
pub mod vector;

/// Contains traits for `StaticBitVec`, `DynBitVec` and `DynBitTree`
pub mod traits;

/// Module providing commonly used utility functions
pub mod utils;

/// Contains implementation of AVL tree
pub mod avl_tree;

/// Configuration for command line arguments.
pub struct Config {
    /// Which algorithm to use. Options are `bv` and `bp`
    pub algo: String,
    /// name of file with input commands
    pub file_in: String,
    /// name of file to write results to
    pub file_out: String,
}

impl Config {
    /// Create new Configuration instance based on arguments passed
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() <= 3 {
            return Err("Usage with parameters is `[bv|bp] input_file output_file`");
        }

        let algo = args[1].clone();

        if algo != "bv" && algo != "bp" {
            return Err("algo needs to be either `bp` or `bv`");
        }

        let file_in = args[2].clone();
        let file_out = args[3].clone();

        Ok(Config {
            algo,
            file_in,
            file_out,
        })
    }
}
