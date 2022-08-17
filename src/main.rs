#![allow(unused_mut)]

use confertus::commands;
use confertus::config::Config;
use confertus::{BitSize, DynBitVec, DynamicBitVector, StaticBitVec};
use std::env;
use std::process;
use std::time::{Duration, Instant};

// use std::mem::size_of;
//
// macro_rules! show_size {
//     (header) => {
//         println!("{:<7} size in bytes  {:>4}    {}", "Type", "T", "Option<T>");
//     };
//     ($t:ty) => {
//         println!(
//             "{:<22} {:4} {:4}",
//             stringify!($t),
//             size_of::<$t>(),
//             size_of::<Option<$t>>()
//         )
//     };
// }

/// TODO
/// - [x] Static Bit Vector
/// - [x] Some kind of self-balancing binary tree (AVL / Red-Black / ...)
/// - [ ] Balanced Parenthesis
/// - [ ] Extending `LeafValue` container
/// - [ ] BP with Range-Min-Max-Tree
fn main() -> Result<(), &'static str> {
    #[cfg(debug_assertions)]
    {
        dbg!(u64::MAX.rank(true, 0));
        dbg!(u64::MAX.rank(true, 64));
        dbg!(cfg!(target_arch = "x86"));
        dbg!(cfg!(target_arch = "x86_64"));
        dbg!(cfg!(target_feature = "bmi1"));
        dbg!(cfg!(target_feature = "bmi2"));
        println!("{}", 15u64.select(false, 2));
        println!("{}", 15u64.select(true, 2));
    }

    // time measured and duration with nanosecond precision
    let mut time_total: Duration = Duration::from_millis(0);
    let mut last_timestamp_cont: Instant = Instant::now();
    let mut dbv = DynamicBitVector::new();

    // println!("{}", u32::MAX);
    // println!("{}", i32::MAX);
    // show_size!(header);
    // show_size!(usize);
    // show_size!(isize);
    // show_size!(i32);
    // show_size!(u32);
    // show_size!(Node);
    // show_size!(Leaf);
    // show_size!(DynamicBitVector);
    // show_size!(Vec<Node>);
    // show_size!(Vec<Leaf>);
    // show_size!(u64);
    // show_size!(u128);
    // show_size!(u8);

    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    commands::write_file(&config.file_out, "").unwrap();

    // let contents = fs::read_to_string(config.file_in.clone())
    //     .expect(&format!("Something went wrong reading the file '{}'", config.file_in));
    // println!("{}", contents);

    if config.algo == "bv" {
        if let Ok(mut lines) = commands::read_lines(config.file_in) {
            if let Some(Ok(first)) = lines.next() {
                #[cfg(debug_assertions)]
                println!("{:?}", first);
                let mut idx = first.parse::<usize>().unwrap();
                #[cfg(debug_assertions)]
                println!("{:?}", idx);
                for (i, line) in lines.enumerate() {
                    if idx > 0 {
                        match line.as_ref().map(String::as_ref) {
                            Ok("0") => dbv.push(false),
                            Ok("1") => dbv.push(true),
                            Ok(val) => panic!("unexpected value: '{val}'"),
                            _ => panic!("unexpected value"),
                        }
                        idx -= 1;
                    } else if let Ok(comm) = line {
                        let command: Vec<&str> =
                            comm.split(' ').filter(|&x| !x.is_empty()).collect();
                        #[cfg(debug_assertions)]
                        println!("{:?}", command);
                        // execute vector commands
                        match command[0] {
                            "insert" => {
                                let index = command[1].parse::<usize>().unwrap();
                                let bit = command[2] != "0";
                                dbv.insert(index, bit)?;
                            }
                            "delete" => {
                                let index = command[1].parse::<usize>().unwrap();
                                dbv.delete(index)?;
                            }
                            "flip" => {
                                let index = command[1].parse::<usize>().unwrap();
                                dbv.flip(index);
                            }
                            "rank" => {
                                let bit = command[1] != "0";
                                let index = command[2].parse::<usize>().unwrap();
                                let rank = dbv.rank(bit, index);

                                time_total += Instant::now().duration_since(last_timestamp_cont);
                                commands::append_file(&config.file_out, rank)?;
                                last_timestamp_cont = Instant::now();
                            }
                            "select" => {
                                let bit = command[1] != "0";
                                let index = command[2].parse::<usize>().unwrap();
                                let sel = dbv.select(bit, index);

                                time_total += Instant::now().duration_since(last_timestamp_cont);
                                commands::append_file(&config.file_out, sel)?;
                                last_timestamp_cont = Instant::now();
                            }
                            _ => panic!(
                                "unrecognized command in file {} at line {i}: {}",
                                config.file_out,
                                command.join(" ")
                            ),
                        }
                    }
                }
            }
        }
    } else if config.algo == "bp" {
        // algo == bp
        if let Ok(lines) = commands::read_lines(config.file_in) {
            for line in lines.flatten() {
                // execute tree commands
                let command: Vec<&str> = line.split(' ').collect();
                match command[0] {
                    "deletenode" => println!("deleting ... {:?}", command),
                    "insertchild" => println!("inserting ... {:?}", command),
                    "child" => println!("child ... {:?}", command),
                    "subtree" => println!("subtree ... {:?}", command),
                    "parent" => println!("parent ... {:?}", command),
                    _ => panic!("unrecognized command in file"),
                }
            }
            println!("This didn't do more than parsing the file actually ...");
        }
    }
    time_total += Instant::now().duration_since(last_timestamp_cont);
    print_results(&config.algo, time_total, dbv);
    Ok(())
}

fn print_results<B>(algo: &str, time: Duration, space: B)
where
    B: BitSize,
{
    println!(
        "RESULT algo={algo} name<Felix Karg> time=<{:?}>[ms] space=<{}>[bits]",
        time.as_millis(),
        space.bitsize_full()
    );
    // println!("RESULTS");
}

/// Apparently it's a unit test simply by being in `main.rs`
#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn exploration() {
        assert_eq!(2 + 2, 4);
    }
}
