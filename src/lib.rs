// deactivate later on again
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_variables)]

/// Module for parsing and building cli commands and args
pub mod commands;

/// Trait definitions, particularly for [`StaticBitVec`] and [`DynBitVec`].
pub mod traits;

/// Actual implementation of dynamic bit vector with AVL Tree
pub mod dynamic_vector;

/// Configuration for command line arguments
pub mod config;

/// Implementation of [`StaticBitVec`] for primitive types ([`u64`], [`u128`], ...)
mod primitive_static;

// /// Module providing commonly used utility functions
// pub mod utils;

// /// Static bit vector implementation: `SBitVec` used as Leaf for dynamic bit vectors `DynBitV`
// /// (incomplete)
// pub mod static_vector;
//
// /// Dynamic Bit vector implementation for `Vec`: `V`
// /// (incomplete)
// pub mod vector;
//
// /// First implementation approach using AVL trees
// /// (incomplete)
// pub mod avl_tree;

mod diff;
mod leaf;
mod node;

#[doc = include_str!("../README.md")]
pub use crate::{commands::*, config::*, dynamic_vector::*, traits::*};
