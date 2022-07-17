# confertus
[![Build Status](https://app.travis-ci.com/fkarg/confertus.svg?branch=main)](https://app.travis-ci.com/fkarg/confertus)

Implementation of succinct bit vectors for lecture 'advanced datastructures' at KIT. Find [code documentation here][docs].

## Usage
General usage is `confertus [bv|bp] input_file output_file` (if you were to
install it via `cargo install --path .`).
- `bv` and `bp` are two different algorithms/datastructures used/tested, where
  `bv` is short for bitvector, and `bp` for balanced parantheses.
- `input_file` is a file containing a number of line-by-line commands.
    - For `bv`, the first line specifies a number `n` of elements to push, and
      the following `n` lines (being `1` or `0`) the bit to insert.
    - Example input files can be found at the [lecture page](https://algo2.iti.kit.edu/4264.php).
- `output_file` may or may not exist beforehand, but will be overwritten if it does.


## Commands
Available commands, depending on selected algorithm:

### Dynamic Bitvector: algo `bv`
- `insert i [0|1]` insert a 0 or 1 at the i-th position of the bit vector
- `delete i` delete the i-th bit
- `flip i` flip the i-th bit
- `rank [0|1] i` write rank0 or rank1 up to position i to the output file
- `select [0|1] i` write select0 or select1 for the i-th occurrence to the output file

### Dynamic Tree datastructure (via Balanced Parentheses): algo `bp`
- `deletenode v` delete node v
- `insertchild v i k` **TODO: check lecture 05**
- `child v i` write i-th child of v to output file
- `subtre size v` write subtree size of v (including v) to output file
- `parent v` write parent of v to output file

(more information about supported operations can also be found in the
[documentation][docs], specifically on
[traits][traits] and the specific implementations for [`bv`][bv] and `bp`)

## Install Rust and Execute
Install `rustup` via your distributions package manager, or, according to [the
official rust website](https://www.rust-lang.org/learn/get-started) via:
```sh
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ # Ensure that ~/.local/bin is in your PATH.
$ # now we need to set the default toolchain
$ rustup default stable

$ # Or, in case of a previous rust install, update with:
$ rustup update

$ # Build and run with (debug)
$ cargo run <args>

$ # Run tests
$ cargo test

$ # Build and run (optimized)
$ RUSTFLAGS="-C target-cpu=native" cargo run --release <args>
```

## Dependencies
- [`either`][either]: Provides the `Either`-datatype. Saves about 10min of
  implementing it manually.

## Development-Dependencies
These are dependencies required to execute tests, not for running the binary itself.

- `pretty_assertions`: visualization of differences in failed assertions, useful for debugging
- `quickcheck`: rust-reimplementation of popular eponymous haskell-based,
  strongly heuristic property-based fuzzing testing library
- `quickcheck_macros`: additional macros for `quickcheck`.
- `test-case`: macros for generating parametricized tests (unused?)
- `rand`: useful access to a random number generator for tests. It has been
  suggested for integration in `std`, but that hasn't happened yet.

I recommend running `cargo watch` on a terminal nearby during active
development. It runs `cargo check` on filechange.


[docs]: https://www.fkarg.me/confertus/docs/confertus/
[traits]: https://www.fkarg.me/confertus/docs/confertus/traits/index.html
[bv]: https://www.fkarg.me/confertus/docs/confertus/dynamic_vector/struct.DynamicBitVector.html
[either]: https://docs.rs/either/latest/either/index.html
