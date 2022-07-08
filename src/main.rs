#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::env;
use std::process;
// use std::fs;
use confertus::avl_tree::AVL;
use confertus::commands;
use confertus::config::Config;

use std::mem::size_of;

macro_rules! show_size {
    (header) => {
        println!("{:<22} {:>4}    {}", "Type", "T", "Option<T>");
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
                let all = first.parse::<usize>().unwrap();
                let mut idx = first.parse::<usize>().unwrap();
                println!("{:?}", idx);
                let size = idx / 64;
                // let mut avl: AVL = AVL::new_with_capacity(size);
                let mut avl: AVL = AVL::empty();
                let mut arr: Vec<bool> = vec![];
                for line in lines {
                    if idx > 0 {
                        match line.as_ref().map(String::as_ref) {
                            Ok("0") => avl.push(false),
                            Ok("1") => avl.push(true),
                            // Ok("0") => avl.insert(all - idx, false),
                            // Ok("1") => avl.insert(all - idx, true),
                            Ok(val) => panic!("unexpected value: '{val}'"),
                            _ => panic!("unexpected value"),
                        }
                        // if let Ok("1") = line {
                        //     // print!("{}", bit);
                        //     // insert bit in array/vector.
                        //     arr.push(true);
                        // } else if let Ok("0") = line {
                        //     arr.push(false);
                        // }
                        idx -= 1;
                    } else {
                        if let Ok(comm) = line {
                            let command: Vec<&str> = comm.split(' ').collect();
                            // println!("{:?}", command);
                            // execute vector commands
                            match command[0] {
                                // "insert" => println!("inserting ..."),
                                "insert" => {
                                    println!("{:?}", command);
                                    avl = commands::insert(avl, command);
                                }
                                "delete" => println!("deleting ..."),
                                "flip" => println!("flipping ..."),
                                "rank" | "select" => println!("requesting ... {:?}", command),
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
