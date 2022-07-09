// deactivate later on again
#![allow(dead_code)]
#![allow(unused_mut)]
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

/// First implementation approach using AVL trees
pub mod avl_tree;

/// Actual implementation of dynamic bit vector with AVL Tree
pub mod dynamic_vector;

/// Configuration for command line arguments
pub mod config;

mod diff;
mod leaf;
mod node;

#[doc = include_str!("../README.md")]
pub use crate::{
    avl_tree::*, commands::*, config::*, dynamic_vector::*, static_vector::*, traits::*, utils::*,
    vector::*,
};
