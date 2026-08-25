// import all of the things 
use fuse_mt::*;
use std::time::Duration;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::fs::File;
use std::time::SystemTime;
use std::collections::VecDeque;
use std::ffi::{CStr, CString, OsStr, OsString};
use super::blocks::{StartBlock, DataBlock};
use super::driveActions::*;


pub struct WeirdFileSystem{
	target : OsString,
}


impl WeirdFileSystem{
	pub fn new(os : OsString) -> Self{
		Self{
			target : os,
		}
	}
}





