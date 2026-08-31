use super::super::blocks::{StartBlock, DataBlock, RawBlock};
use super::super::results::*;
use super::*;
use std::{
    fs::File,
    os::unix::fs::FileExt,
	ffi::{OsStr,OsString}
};
use fuse_mt::{ResultEmpty,DirectoryEntry,FileType,ResultReaddir};


fn modify_directory_contents(
	drive_file : &File,
	directory_entry_index : u32,
	new_directory_entry_data : u32,
	directory_start_block : &mut  StartBlock,
) -> ResultEmpty {
	
	
	let mut data_block_vec = data_block_actions::get_data_block_vec_from_start_block(drive_file,directory_start_block.clone())?;
	let new_data = new_directory_entry_data.to_ne_bytes();
	let data_offset : usize = usize::try_from(directory_entry_index*4).unwrap();


	
	let modified_data_block_vec = data_block_actions::data_block_vector_set_data_from_data_vector(
		&mut data_block_vec,
		&mut new_data.to_vec(),
		data_offset % 248,
		data_offset / 248
	).unwrap();
	block_actions::bulk_data_block_write(drive_file,modified_data_block_vec);
	Ok(())
}
pub fn remove_directory_entry(
	drive_file : &File,
	start_block_entry_index_to_remove : u32,
	directory_start_block : &mut StartBlock
) -> ResultEmpty {
	let data_block_vec = data_block_actions::get_data_block_vec_from_start_block(drive_file,directory_start_block.clone())?;
	let directory_entry_vector = data_block_actions::get_directory_index_vec_from_data_block_vec(data_block_vec)?;
	for i in 0..directory_entry_vector.len(){
		if directory_entry_vector[i] == start_block_entry_index_to_remove{
			modify_directory_contents(drive_file,u32::try_from(i).unwrap(),u32::MAX,directory_start_block);
			return Ok(())
		}
	}
	Err(libc::ENOENT)

}
pub fn append_directory_content(
	drive_file : &File,
	new_directory_entry_data : u32,
	directory_start_block : &mut StartBlock
) -> ResultEmpty{
	let directory_data = data_block_actions::get_directory_data_from_start_block_index(drive_file,directory_start_block.get_block_index().clone())?;
	for i in 0..directory_data.len(){
		if directory_data.get(i).unwrap().clone() == u32::MAX{
			modify_directory_contents(drive_file, u32::try_from(i).unwrap(),new_directory_entry_data,directory_start_block)?;
			return Ok(())	
		}
	}
	// if you are here all avalible slots are taken
	// we must expand
	// THE MONGLES
	// WE ARE THE EXEPTION

	file_utils::expand_file(drive_file,directory_start_block.clone(),1,true)?;
	let directory_data = data_block_actions::get_directory_data_from_start_block_index(drive_file,directory_start_block.get_block_index().clone())?;
	for i in 0..directory_data.len(){
		if directory_data.get(i).unwrap().clone() == u32::MAX{
			modify_directory_contents(drive_file, u32::try_from(i).unwrap(),new_directory_entry_data,directory_start_block)?;
			return Ok(())	
		}
	}
	Ok(())
}
pub fn get_directory_entry_vec_from_start_block_index(
	drive_file : &File,
	start_block_index : u32
) -> ResultReaddir {
	let directory_indexes = data_block_actions::get_directory_data_from_start_block_index( drive_file, start_block_index)?;
	let mut directory_entries : Vec<DirectoryEntry> = Vec::new();
	for index in directory_indexes {
		let current_start_block = block_actions::start_block_read(drive_file,index)?;
		directory_entries.push(
			DirectoryEntry{
				name : OsString::from(current_start_block.get_name()),
				kind : match current_start_block.get_file_type() {
					1 => FileType::NamedPipe,
					2 => FileType::CharDevice,
					3 => FileType::Directory,
					4 => FileType::RegularFile,
					5 => FileType::Symlink,
					6 => FileType::Socket,
					_ => FileType::RegularFile
				}
			}
		)
	}
	Ok(directory_entries)
	
}
