use super::super::blocks::{StartBlock, DataBlock};
use super::super::results::*;
use super::*;
use std::{
    fs::File,
    os::unix::fs::FileExt,
	ffi::{OsStr,OsString}
};
use fuse_mt::ResultEmpty;



pub fn deallocate_data_blocks_from_end(
	drive_file : &File,
	start_block : &mut StartBlock,
	number_of_blocks : usize
) -> ResultEmpty {
	let mut data_block_vector = data_block_actions::get_data_block_vec_from_start_block(drive_file,start_block.clone())?;
	if data_block_vector.len() >= number_of_blocks{
		return Err(libc::EDOM)
	}

	for _i in 0..number_of_blocks {
		let current_block = match data_block_vector.pop(){
			Some(block) => block,
			None => return Err(libc::EIO)
		};
		block_actions::direct_block_deletion(drive_file,current_block.get_block_index().clone())?;
		
	}
	let mut new_last_block = data_block_vector.pop().unwrap();
	new_last_block.set_next_block_index(u32::MAX);
	Ok(())
}


fn deallocate_all_blocks(
	drive_file : &File,
	start_block : &mut StartBlock
) -> ResultEmpty {
	let data_block_vec = data_block_actions::get_data_block_vec_from_start_block(drive_file,start_block.clone())?;
	let mut indexes_to_delete : Vec<u32> = Vec::new();
	for block in data_block_vec{
		indexes_to_delete.push(block.get_block_index().clone());
		
	}
	indexes_to_delete.push(start_block.get_block_index().clone());
	for index in indexes_to_delete{
		block_actions::direct_block_deletion(drive_file,index);
	}
	Ok(())
}

pub fn delete_file(
	drive_file : &File,
	start_block : &mut StartBlock,
	parent_start_block : &mut StartBlock
) -> ResultEmpty {
	directory_actions::remove_directory_entry(drive_file,start_block.get_block_index().clone(),parent_start_block);
	deallocate_all_blocks(drive_file,start_block);
	Ok(())
}


pub fn delete_all_files_in_directory(
	drive_file : &File,
	directory_start_block: &mut StartBlock
) -> ResultEmpty {
	let child_start_block_index_vector = data_block_actions::get_directory_data_from_start_block_index(drive_file, directory_start_block.get_block_index().clone())?;
	for block_index in child_start_block_index_vector{
		delete_file(
			drive_file,
			&mut block_actions::start_block_read(drive_file,block_index)?,
			directory_start_block,
		)?;
	}
	Ok(())
}

pub fn delete_directory (
	drive_file : &File,
	directory_start_block : &mut StartBlock,
	parent_start_block : &mut StartBlock
) -> ResultEmpty {
	delete_all_files_in_directory(
		drive_file,
		directory_start_block
	)?;
	delete_file(
		drive_file,
		directory_start_block,
		parent_start_block
	)?;
	Ok(())
}
