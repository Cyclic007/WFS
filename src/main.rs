// Main Entry Point :: A fuse_mt test program.
//
// Copyright (c) 2016-2022 by William R. Fraser
//












#![deny(rust_2018_idioms)]

#![doc = include_str!("../docs/readme.html")]

use std::env;
use std::ffi::{OsStr, OsString};
use filesystem::WeirdFileSystem;

/// Contains the actual filesystem Implementaion
mod filesystem;
/// contains the block definintions and setters and getters
mod blocks;
/// contains the functions to implement the filesystem
mod drive_actions;
/// the file handle implementaion 
mod handles;
/// the location where all of the reururn types are
mod results;



#[doc(hidden)]
struct ConsoleLogger;
#[doc(hidden)]
impl log::Log for ConsoleLogger {
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        println!("{}: {}: {}", record.target(), record.level(), record.args());
    }

    fn flush(&self) {}
}
#[doc(hidden)]
static LOGGER: ConsoleLogger = ConsoleLogger;
#[doc(hidden)]
fn main() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);

    let args: Vec<OsString> = env::args_os().collect();

    if args.len() != 3 {
        println!("usage: {} <target> <mountpoint>", &env::args().next().unwrap());
        std::process::exit(-1);
    }

    let filesystem =  filesystem::WeirdFileSystem::new(args[1].clone());

    let fuse_args = [OsStr::new("-o"), OsStr::new("fsname=WFS",)];

    fuse_mt::mount(fuse_mt::FuseMT::new(filesystem, 1), &args[2], &fuse_args[..]).expect("ahhh");
}
