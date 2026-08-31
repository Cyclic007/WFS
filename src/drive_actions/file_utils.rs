use super::super::blocks::{StartBlock, DataBlock};
use super::super::results::*;
use super::*;
use std::{
    fs::File,
    os::unix::fs::FileExt,
	ffi::{OsStr,OsString}
};
use fuse_mt::ResultEmpty;



pub fn expand_file (
	drive_file : &File,
	start_block : StartBlock,
	number_of_additonal_blocks : usize,
	is_directory : bool
) -> ResultEmpty {
	let current_data_block_index = start_block.get_first_data_block_index().clone();
	let mut current_data_block = block_actions::data_block_read(drive_file,current_data_block_index)?;
	let current_data_block_index = current_data_block.get_next_block_index().clone();
	while current_data_block_index != u32::MAX{
		let current_data_block = block_actions::data_block_read(drive_file,current_data_block_index)?;
		let _current_data_block_index = current_data_block.get_next_block_index();
	}

	let new_block_indexes = write_utils::find_next_empty_blocks(drive_file,number_of_additonal_blocks)?;
	current_data_block.set_next_block_index(new_block_indexes[0]);
	block_actions::data_block_write(drive_file,current_data_block);
	for i in 1..new_block_indexes.len(){
		block_actions::data_block_write(
			drive_file,
			DataBlock::new(
				new_block_indexes[i],
				match is_directory{
					true => [225;248],
					false => [0;248]
				},
				 new_block_indexes[i+1]	
			)
		)?;
		if i ==  new_block_indexes.len()-1{
			block_actions::data_block_write(drive_file,DataBlock::new_plain(new_block_indexes[i],is_directory));
			break;
		}
	}
	Ok(())
}

pub fn read_file(
	drive_file : &File,
	start_block : &mut StartBlock,
	offset : usize,
	size : usize
) -> ResultDataVector {
	println!("reading {} bytes",size);
	let full_data_vector = data_block_actions::get_data_vector_from_start_block(drive_file,start_block)?;
	let mut full_data_iter = full_data_vector.iter();
	let mut returned_data_vector : Vec<u8> = Vec::with_capacity(size);
	for _i in offset..(offset+size){
		match full_data_iter.next(){
			Some(byte) => returned_data_vector.push(byte.clone()),
			None => return Ok(returned_data_vector)
		}
	} 
	Ok(returned_data_vector)
	
}


pub fn reduce_file_size(
	drive_file : &File,
	start_block : &mut StartBlock,
	new_size : u64
) -> ResultEmpty {
	let inital_file_size = start_block.get_size().clone();
	if inital_file_size < new_size{
		return Err(libc::EDOM)
	}
	let num_blocks_to_remove : usize =usize::try_from(new_size/248-inital_file_size/248).unwrap();

	deletion_utils::deallocate_data_blocks_from_end(
		drive_file,
		start_block,
		num_blocks_to_remove
	)?;


	let number_of_extra_bytes_to_remove = (new_size-inital_file_size) as usize;
	let mut new_data_block_vector = data_block_actions::get_data_block_vec_from_start_block(drive_file,start_block.clone())?;
	let mut last_block = new_data_block_vector.pop().unwrap();
	let mut last_block_new_data : Vec<u8> = Vec::new();
	for _i in 0..248-number_of_extra_bytes_to_remove{
		last_block_new_data.push(0)
	}
	last_block.set_data(last_block_new_data,248-number_of_extra_bytes_to_remove).unwrap();
	new_data_block_vector.push(last_block);
	let mut block_index_vector : Vec<u32> = Vec::new();
	for block in new_data_block_vector.clone(){
		block_index_vector.push(block.get_block_index().clone());
	}
	data_block_actions::data_block_vector_write(drive_file,new_data_block_vector,block_index_vector)?;
	Ok(())
}
