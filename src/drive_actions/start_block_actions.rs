use super::super::blocks::{StartBlock, DataBlock, RawBlock};
use super::super::results::*;
use super::*;
use std::{
    fs::File,
    os::unix::fs::FileExt,
	ffi::{OsStr,OsString}
};




fn get_start_block_from_name_and_directory_start_block_index(
	drive_file: &File,
	directory_start_block_index : u32,
	name : OsString,
) -> ResultStartBlockOption  {
	let directory_data = data_block_actions::get_directory_data_from_start_block_index(drive_file,directory_start_block_index)?;
	for index in directory_data{
		let current_start_block = block_actions::start_block_read(drive_file, index)?;
		if current_start_block.clone().get_name().eq(&name.clone().into_string().unwrap()){
			return Ok(Some(current_start_block))
		}
	}
	return Ok(None)
	
}


pub fn get_start_block_from_path(
	drive_file : &File,
	path : &OsStr
) -> ResultStartBlockOption{
	
	if path.eq("/"){
		return Ok(Some(block_actions::start_block_read(drive_file,1)?))
	}
	
	let name_list : Vec<&str> = path.to_str().unwrap().splitn(20, "/").collect();
	let mut current_block_index = 1;
	for name in name_list.clone(){
		let current_start_block = match get_start_block_from_name_and_directory_start_block_index ( drive_file , current_block_index , OsString::from(name))?{
			Some(block) => block,
			None => block_actions::start_block_read(drive_file,1)?
			
			
		};
		if current_start_block.get_name().eq(name_list[name_list.clone().len()-1]){
			return Ok(Some(current_start_block))
		}
		current_block_index = current_start_block.get_block_index().clone();
	}
	return Ok(None)
}
