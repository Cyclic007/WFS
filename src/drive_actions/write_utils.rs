use super::super::blocks::{StartBlock, DataBlock, RawBlock};
use super::super::results::*;
use super::*;
use std::{
    fs::File,
    os::unix::fs::FileExt,
	ffi::{OsStr,OsString}
};




pub fn write_new_empty_directory (
	drive_file : &File,
	start_block : &mut StartBlock,
) -> ResultStartBlock{
	let empty_blocks = drive_utils::find_next_empty_blocks(drive_file,2)?;
	start_block.set_block_index(empty_blocks[0]);
	start_block.set_first_data_block_index(empty_blocks[1]);
	block_actions::data_block_write(drive_file,DataBlock::new_plain_dir(empty_blocks[1]));
	block_actions::start_block_write(drive_file, start_block.clone());
	return Ok(start_block.clone())
}

fn write_new_empty_file (
	drive_file : &File,
	start_block : &mut StartBlock,
) -> ResultStartBlock{
	let empty_blocks = drive_utils::find_next_empty_blocks(drive_file,2)?;
	start_block.set_block_index(empty_blocks[0]);
	start_block.set_first_data_block_index(empty_blocks[1]);
	block_actions::data_block_write(drive_file,DataBlock::new_plain_file(empty_blocks[1]));
	block_actions::start_block_write(drive_file, start_block.clone());
	return Ok(start_block.clone())
}

pub fn create_empty_directory(
	drive_file : &File,
	parent_start_block : &mut StartBlock,
	child_start_block : &mut StartBlock
	
) -> ResultBlockIndex {
	let new_child_start_block = write_new_empty_directory(drive_file,&mut child_start_block.clone())?;
	directory_actions::append_directory_content(drive_file,new_child_start_block.get_block_index().clone(),parent_start_block);
	Ok(child_start_block.get_block_index().clone())
}

pub fn create_empty_file(
	drive_file : &File,
	parent_start_block : &mut StartBlock,
	child_start_block : &mut StartBlock
) -> ResultBlockIndex {
	let new_child_start_block = write_new_empty_file(drive_file,&mut child_start_block.clone())?;
	directory_actions::append_directory_content(drive_file,new_child_start_block.get_block_index().clone(),parent_start_block);
	Ok(child_start_block.get_block_index().clone())
}

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

pub fn write_to_file(
	drive_file : &File,
	offset : usize,
	data : &mut Vec<u8>,
	start_block : &mut StartBlock,
	
) -> ResultSize{


	//first need to check if the file size needs to be changed and if it needs more blocks
	let total_size = start_block.get_size();
	let data_size = data.len();
	let final_changed_byte  = offset + data_size;
	let total_inital_block_size = (f64::try_from(u32::try_from(total_size.clone()).unwrap()).unwrap() / 248.0).ceil();
	let final_modified_block_size : usize = (f64::try_from(u32::try_from(final_changed_byte).unwrap()).unwrap() / 248.0).ceil() as usize;
	if total_inital_block_size < final_modified_block_size as f64 {
		start_block.set_size(u64::try_from(final_changed_byte).unwrap());
		block_actions::start_block_write(drive_file,start_block.clone());
		file_utils::expand_file(
			drive_file,
			start_block.clone(),
			final_modified_block_size - total_inital_block_size as usize,
			match start_block.get_file_type(){
				3 => true,
				_ => false,
			}
		);
	}
	let current_full_data_block_vector : Vec<DataBlock> = data_block_actions::get_data_block_vec_from_start_block(drive_file,start_block.clone())?;
	let mut modified_data_block_vector : Vec<DataBlock> = Vec::new(); 
	for i in offset / 248 .. final_changed_byte/248+1{
		modified_data_block_vector.push(current_full_data_block_vector.get(i).unwrap().clone());
	}
	let first_block_offset = offset % 248;
	
	let blocks_to_be_written : Vec<DataBlock> = data_block_actions::data_block_vector_set_data_from_data_vector(&mut modified_data_block_vector,data,first_block_offset,0)?;
	
	for block in blocks_to_be_written{
		block_actions::data_block_write(drive_file,block);
	}
	Ok(usize::try_from(start_block.clone().get_size().clone()).unwrap())



}
