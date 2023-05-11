# ApeMBR
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE.txt)
[![Crates.io](https://img.shields.io/crates/v/ape-mbr)](https://crates.io/crates/ape-mbr)
[![Documentation](https://docs.rs/ape-mbr/badge.svg)](https://docs.rs/ape-mbr)
[![APE](https://img.shields.io/badge/-APE-%2359118e)](https://openapeshop.org/)
## *simple crate to interface between a disk and it's partitions*

This crate is especially designed to provide an interface between a disk
and a file system library, where both are able to implement embedded_io.

Dead simple, as it should be.

## Usage

This crate can be used by adding `ape-mbr` to the dependencies in your
project's `Cargo.toml`.

```toml
[dependencies]
ape-mbr = "0.1.0"
```

## Examples

Here's `ape-mbr` being coupled with `ape-fatfs`

```rust
use std::io::prelude::*;
use ape_fatfs::{
    fs::{
        FsOptions,
        FileSystem,
    },
    io::{
        StdIoWrapper
    }
};

use ape_mbr::{
    PartitionId,
    MBR,
};

fn main() {
    // Initialize the MBR
    let img_file = std::fs::OpenOptions::new().read(true).write(true)
        .open("test.img").unwrap();

    let img_file = StdIoWrapper::new(img_file);
   
    let mut mbr = MBR::new(img_file).unwrap();
    let mut p1 = mbr.get_partition(PartitionId::One).unwrap();
    
    let fs = FileSystem::new(p1, FsOptions::new()).unwrap();
    let root_dir = fs.root_dir();

    // Write a file
    root_dir.create_dir("foo").unwrap();
    let mut file = root_dir.create_file("foo/hello.txt").unwrap();
    file.truncate().unwrap();
    file.write_all(b"Hello World!").unwrap();

    // Read a directory
    let dir = root_dir.open_dir("foo").unwrap();
    for r in dir.iter() {
        let entry = r.unwrap();
        println!("{}", entry.file_name());
    }
}
```
