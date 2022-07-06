# confertus
[![Build Status](https://app.travis-ci.com/fkarg/confertus.svg?branch=main)](https://app.travis-ci.com/fkarg/confertus)

Implementation of succinct bit vectors for lecture 'advanced datastructures' at KIT. Find [code documentation here](https://www.fkarg.me/confertus/docs/confertus/)

Dynamic Bitvector with operations:
- access
- `insert i [0|1]` insert a 0 or 1 at the i-th position of the bit vector
- `delete i` delete the i-th bit
- `flip i` flip the i-th bit
- `rank [0|1] i` write rank0 or rank1 up to position i to the output file
- `select [0|1] i` write select0 or select1 for the i-th occurrence to the output file

Dynamic Tree datastructure (via Balanced Parentheses) with operations:
- `deletenode v` delete node v
- `insertchild v i k` **TODO: check lecture 05**
- `child v i` write i-th child of v to output file
- `subtre size v` write subtree size of v (including v) to output file
- `parent v` write parent of v to output file


## Install Rust
according to [the official rust website](https://www.rust-lang.org/learn/get-started) you install rust via:
```sh
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ # Ensure that ~/.local/bin is in your PATH.
$ # now we need to set the default toolchain
$ rustup default stable
$ # Build and run with (debug)
$ cargo run <args>
$ # Build and run (optimized)
$ cargo run --release <args>
```

Does not depend on external libraries, but uses `std`.
