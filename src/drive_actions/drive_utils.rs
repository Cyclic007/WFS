use super::super::blocks::{StartBlock, DataBlock, RawBlock};
use super::super::results::*;
use super::*;
use std::{
    fs::File,
    os::unix::fs::FileExt,
	ffi::{OsStr,OsString}
};

pub fn find_next_empty_blocks(
	drive_file : &File,
	number_of_blocks : usize,
) -> ResultBlockIndexVector {
	let mut index_vec : Vec<u32> = Vec::with_capacity(number_of_blocks);
	for i in 0..u32::MAX{
		if block_actions::data_block_read(drive_file,i).unwrap().get_block_index().clone() != i{
			index_vec.push(i);
			if index_vec.len() == number_of_blocks{
				return Ok(index_vec)
			}
		}
	}
	return Ok(index_vec)
}
