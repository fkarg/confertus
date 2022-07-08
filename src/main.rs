#![allow(unused_mut)]

use std::env;
use std::process;
// use std::fs;
use confertus::avl_tree::AVL;
use confertus::commands;
use confertus::config::Config;
use confertus::{DynamicBitVector, Leaf, Node};

use std::mem::size_of;

macro_rules! show_size {
    (header) => {
        println!("{:<7} size in bytes  {:>4}    {}", "Type", "T", "Option<T>");
    };
    ($t:ty) => {
        println!(
            "{:<22} {:4} {:4}",
            stringify!($t),
            size_of::<$t>(),
            size_of::<Option<$t>>()
        )
    };
}

/// TODO
/// - [ ] Static Bit Vector
/// - [ ] Some kind of self-balancing binary tree (AVL / Red-Black / ...)
/// - [ ] Range-Min-Max-Tree
fn main() {
    show_size!(header);
    show_size!(usize);
    show_size!(isize);
    show_size!(AVL);
    show_size!(Box<AVL>);
    show_size!(i32);
    show_size!(u32);
    show_size!(Node);
    show_size!(Leaf);
    show_size!(DynamicBitVector);
    show_size!(Vec<Node>);
    show_size!(Vec<Leaf>);
    show_size!(u64);
    show_size!(u128);
    show_size!(u8);

    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    // let contents = fs::read_to_string(config.file_in.clone())
    //     .expect(&format!("Something went wrong reading the file '{}'", config.file_in));
    // println!("{}", contents);

    if config.algo == "bv" {
        if let Ok(mut lines) = commands::read_lines(config.file_in) {
            if let Some(Ok(first)) = lines.next() {
                println!("{:?}", first);
                let mut idx = first.parse::<usize>().unwrap();
                let mut dbv = DynamicBitVector::new();
                println!("{:?}", idx);
                for line in lines {
                    if idx > 0 {
                        match line.as_ref().map(String::as_ref) {
                            Ok("0") => dbv.push(false),
                            Ok("1") => dbv.push(true),
                            Ok(val) => panic!("unexpected value: '{val}'"),
                            _ => panic!("unexpected value"),
                        }
                        idx -= 1;
                    } else {
                        if let Ok(comm) = line {
                            let command: Vec<&str> = comm.split(' ').collect();
                            println!("{:?}", command);
                            // execute vector commands
                            match command[0] {
                                // "insert" => println!("inserting ..."),
                                "insert" => {
                                    dbv = commands::insert(dbv, command);
                                }
                                "delete" => {
                                    dbv = commands::delete(dbv, command);
                                }
                                "flip" => {
                                    dbv = commands::flip(dbv, command);
                                }
                                "rank" => {
                                    dbv = commands::rank(dbv, command);
                                }
                                "select" => {
                                    dbv = commands::select(dbv, command);
                                }
                                _ => panic!("unrecognized command in file"),
                            }
                        }
                    }
                }
            }
        }
    } else {
        // algo == bp
        if let Ok(lines) = commands::read_lines(config.file_in) {
            for line in lines {
                if let Ok(comm) = line {
                    // execute tree commands
                    let command: Vec<&str> = comm.split(' ').collect();
                    match command[0] {
                        "deletenode" => println!("deleting ... {:?}", command),
                        "insertchild" => println!("inserting ... {:?}", command),
                        "child" => println!("child ... {:?}", command),
                        "subtree" => println!("subtree ... {:?}", command),
                        "parent" => println!("parent ... {:?}", command),
                        _ => panic!("unrecognized command in file"),
                    }
                }
            }
        }
    }

    print_results();
}

fn print_results() {
    // RESULT algo=bv name<first last name> time=<running time without output in ms> space=<required space in bits>
    // println!("RESULT algo={} name<Felix Karg> time=<{}> space=<{}>");
    println!("RESULTS");
}
