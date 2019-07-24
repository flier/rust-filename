# filename [![travis](https://travis-ci.org/flier/rust-filename.svg?branch=master)](https://travis-ci.org/flier/rust-filename) [![Build status](https://ci.appveyor.com/api/projects/status/jhlk20pjdrx0jh5u?svg=true)](https://ci.appveyor.com/project/flier/rust-filename) [![crate](https://img.shields.io/crates/v/filename.svg)](https://crates.io/crates/filename) [![docs](https://docs.rs/filename/badge.svg)](https://docs.rs/crate/filename/) [![dependency status](https://deps.rs/repo/github/flier/rust-filename/status.svg)](https://deps.rs/repo/github/flier/rust-filename)

Get filename from a raw file descriptor

## Usage

To use `filename` in your project, add the following to your Cargo.toml:

``` toml
[dependencies]
filename = "0.1"
```

## Example

```rust
use filename::file_name;

let f = tempfile::tempfile().unwrap();

println!("tempfile @ {:?}", file_name(&f).unwrap());
```
