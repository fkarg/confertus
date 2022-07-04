use std::env;
use std::process;
// use std::fs;
use confertus::commands;
use confertus::Config;

fn main() {
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
                println!("{:?}", idx);
                for line in lines {
                    if idx > 0 {
                        if let Ok(bit) = line {
                            print!("{}", bit);
                            // insert bit in array/vector.
                        }
                        idx -= 1;
                    } else {
                        if let Ok(comm) = line {
                            let command: Vec<&str> = comm.split(' ').collect();
                            println!("{:?}", command);
                            // execute vector commands
                            match command[0] {
                                "insert" => println!("inserting ..."),
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
        if let Ok(mut lines) = commands::read_lines(config.file_in) {
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
