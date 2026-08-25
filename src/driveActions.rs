// This will define shortcuts for minipulating the data on the drive
use std::{
    fs::File,
    os::unix::fs::FileExt,
    collections::VecDeque,
	ffi::{OsStr,OsString}
};
use super::blocks::{StartBlock, DataBlock, RawBlock};
use fuse_mt::{DirectoryEntry,FileType};


fn get_offset_from_block_index(block_index : u32) -> Result<u64, &'static str>{
	u32::try_from(block_index*256)?
}

fn direct_raw_block_read(
	drive_file : &File,
	block_index : u32,	
) -> Result<RawBlock, &'static str>{
	if block_index == u32::MAX {
		Err("null block index")
	}
	let read_offset = get_offset_from_block_index(block_index);
	let mut read_buffer : [u8 ; 256] = [0 ; 256];
	drive_file.read_exact_at(&mut read_buffer , read_offset)?;
	Ok(
		RawBlock{
			read_buffer
		}
	)

}

fn direct_raw_block_write(
	drive_file : &File,
	block_index : u32,
	block : RawBlock
) -> Result<u32, &'static str>{
	if block_index == u32::MAX {
		Err("null block index")
	}
	let write_offset = get_offset_from_block_index(block_index)?;
	drive_file.write_all_at(&block.data,write_offset);
	Ok(100)
}


pub fn data_block_read(
	drive_file : &File,
	block_index: u32,
) -> Result<DataBlock, &'static str>{
	Ok(
		DataBlock::from(direct_raw_block_read(drive_file,block_index)?)
	)
}

pub fn start_block_read(
	drive_file : &File,
	block_index: u32,
) -> Result<StartBlock, &'static str>{
	Ok(
		StartBlock::from(direct_raw_block_read(drive_file,block_index)?)
	)
}

pub fn data_block_write(
	drive_file : &File,
	block : DataBlock,
) -> Result<u32, &'static str>{
	direct_raw_block_write(
		drive_file,
		block.get_block_index(),
		RawBlock::from(block)
	)
}
pub fn start_block_write(
	drive_file : &File,
	block : StartBlock,
) -> Result<u32, &'static str>{
	direct_raw_block_write(
		drive_file,
		block.get_block_index(),
		RawBlock::from(block)
	)
}


fn get_data_block_vec_from_first_data_block_index(
	drive_file : &File,
	first_block_index : u32,
) -> Result< Vec<DataBlock>, &'static str>{
	let mut block_vec : Vec<DataBlock> = Vec::new(); 
	let first_block = data_block_read(drive_file,first_block_index)?;
	block_vec.push(first_block);
	let mut current_next_index = first_block.get_next_block_index();
	while current_next_index != u32::MAX{
		let current_block = data_block_read(drive_file, current_next_index)?;
		block_vec.push(current_block);
		current_next_index = current_block.get_next_block_index();
	}
	Ok(block_vec)
}


fn get_data_block_vec_from_start_block(
	drive_file : &File,
	start_block : StartBlock
) -> Result< Vec<DataBlock>, &'static str> {
	get_data_block_vec_from_first_data_block_index(
		drive_file,
		start_block.get_first_data_block_index()
	)
}

fn get_data_block_vec_from_start_block_index(
	drive_file : &File,
	start_block_index : u32,
) -> Result< Vec<DataBlock>, &'static str>{
	get_data_block_vec_from_start_block(
		drive_file,
		start_block_read(
			drive_file,
			start_block_index
		)?
	)
}



fn get_data_vec_from_data_block_vec(
	data_block_vec : Vec<DataBlock>
) -> Result< Vec<u8>, &'static str>{
	let mut data_vec : Vec<u8> = Vec::new();
	for block in data_block_vec{
		for byte in block.get_data(){
			data_vec.push(byte)
		}
	}
}


fn get_directory_index_vec_from_data_block_vec(
	data_block_vec : Vec<DataBlock>
) -> Result< Vec<u32>, &'static str>{
	let mut directory_vector : Vec<u32> = Vec::new();
	let data_vec : Vec<u8> = get_data_vec_from_data_block_vec(data_block_vec);
	for directory_chunk in data_vec.chunks_exact(4){
		let directory_index_buffer : u32 = u32::from_ne_bytes(directory_chunk);
		match directory_index_buffer{
			u32::MAX => println!("no more"),
			_ => directory_vector.push(directory_index_buffer)
		}
	}


	Ok(directory_vector)
}



pub fn get_file_data_from_start_block_index(
	drive_file : &File,
	start_block_index : u32
)  -> Result< Vec<u8>, &'static str>{
	get_data_vec_from_data_block_vec(
		get_data_block_vec_from_start_block_index(drive_file,start_block_index)
	)
}


fn get_directory_data_from_start_block_index(
	drive_file : &File,
	start_block_index : u32
) -> Result< Vec<u32>, &'static str>{
	get_directory_index_vec_from_data_block_vec(
		get_data_block_vec_from_start_block_index(drive_file,start_block_index)
	)
}



pub fn get_directory_entry_vec_from_start_block_index(
	drive_file : &File,
	start_block_index : u32
) -> Result<Vec<DirectoryEntry>, &'static str> {
	let directory_indexes = get_directory_data_from_start_block_index( drive_file, start_block_index);
	let mut directory_entries : Vec<DirectoryEntry> = Vec::new();
	for index in directory_indexes {
		let current_start_block = start_block_read(drive_file,index);
		directory_entries.push(
			DirectoryEntry{
				name : current_start_block.get_name(),
				kind : match current_start_block.get_file_type() {
					1 => FileType::NamedPipe,
					2 => FileType::CharDevice,
					3 => FileType::Directory,
					4 => FileType::RegularFile,
					5 => FileType::Symlink,
					6 => FileType::Socket
				}
			}
		)
	}
	Ok(directory_entries)
	
}


fn get_start_block_from_name_and_directory_start_block_index(
	drive_file: &File,
	directory_start_block_index : u32,
	name : OsStr,
) -> Result<Option<StartBlock>, &'static str> {
	let directory_data = get_directory_data_from_start_block_index()?;
	for index in directory_data{
		let current_start_block = start_block_read(drive_file, index)?;
		if current_start_block.get_name().equals(name){
			Ok(current_start_block)
		}
	}
	Ok(())
	
}


pub fn get_start_block_from_path(
	drive_file : &File,
	path : &OsStr
) -> Result<Option<StartBlock>, &'static str>{
	let name_list = path.to_str().splitn(20, "/");
	let mut current_block_index = 0;
	for name in name_list{
		let current_start_block = get_start_block_from_name_and_directory_start_block_index ( drive_file , current_block_index , name);
		if current_start_block.get_name().equals(name_list[name_list.size-1]){
			Ok(current_start_block)
		}
		current_block_index = current_start_block.get_block_index();
	}
	Ok(())
}



