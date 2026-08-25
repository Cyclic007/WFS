// Main Entry Point :: A fuse_mt test program.
//
// Copyright (c) 2016-2022 by William R. Fraser
//

#![deny(rust_2018_idioms)]
mod filesystem;
use std::env;
use std::ffi::{OsStr, OsString};
use filesystem::WeirdFileSystem;
mod blocks;
mod driveActions;
mod handles;

struct ConsoleLogger;

impl log::Log for ConsoleLogger {
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        println!("{}: {}: {}", record.target(), record.level(), record.args());
    }

    fn flush(&self) {}
}

static LOGGER: ConsoleLogger = ConsoleLogger;

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
