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

fn find_next_empty_blocks(
	drive_file : &File,
	number_of_blocks : u32,
) -> Vec<u32> {
	let index_vec : Vec<u32> = Vec::with_capacity(number_of_blocks);
	for i in 0..u32::MAX{
		if data_block_read(drive_file,i).get_block_index() != i{
			index_vec.push(i);
			if index_vec.len() == number_of_blocks{
				return index_vec
			}
		}
	}
}






//This does not add it to a parent directory
pub fn create_new_empty_directory (
	drive_file : &File,
	start_block : StartBlock,
	is_directory : bool
) -> StartBlock {
	let empty_blocks = find_next_empty_blocks(drive_file,2);
	start_block.set_block_index(empty_blocks[0]);
	start_block.set_first_data_block_index(empty_blocks[1]);
	data_block_write(drive_file,DataBlock::new_plain_dir(empty_blocks[1]));
	start_block_write(drive_file, start_block)
}



pub fn expand_file (
	drive_file : &File,
	start_block : StartBlock,
	number_of_additonal_blocks : u32,
	is_directory : bool
) -> Result<u32, &'static str> {
	let current_data_block_index = start_block.get_first_data_block_index();
	let current_data_block = data_block_read(current_data_block_index);
	let current_data_block_index = current_data_block.get_next_data_block_index();
	while current_data_block_index != u32::MAX{
		let current_data_block = data_block_read(current_data_block_index);
		let current_data_block_index = current_data_block.get_next_data_block_index();
	}

	let new_block_indexes = find_next_empty_blocks(drive_file,number_of_additonal_blocks);
	current_data_block.set_next_block_index(new_block_indexes[0]);
	data_block_write(drive_file,current_data_block);
	for i in 1..new_block_indexes.len(){
		data_block_write(
			DataBlock{
				block_index : new_block_indexes[i],
				data : match is_directory{
					true => [225;248],
					false => [0;248]
				},
				next_block_index : new_block_indexes[i+1]	
			}
		);
		if i ==  new_block_indexes.len()-1{
			data_block_write(DataBlock::new_plain(new_block_indexes[i],is_directory))
		}
	}
	Ok(27)
}





//Index vector must be the same size as the data block vector
fn data_block_vector_write(
	drive_file : &File,
	mut data_block_vector : Vec<DataBlock>,
	data_block_index_vector : Vec<u32>
) -> Result<u32, &'static str> {
	if data_block_index_vector.len() != data_block_index_vector{
		Err("number of blocks is not equal to number of indexes")
	}

	
	for i in data_block_index_vector.len(){
		data_block_vector[i].set_block_index(data_block_index_vector[i]);
		if i == data_block_index_vector.len()-1{
			data_block_vector[i].set_next_block_index(u32::MAX);
		}else {
			data_block_vector[i].set_next_block_index(data_block_index_vector[i+1]);
		}
	}
	for block in data_block_vector{
		data_block_write(
			drive_file,
			block
		);
	}
	Ok(42)
	
}







fn data_block_vector_set_data_from_data_vector(
	&mut data_block_vector : Vec<DataBlock>,
	data_vector : Vec<u8>,
	first_block_offset : u64, //Offset to start writeing only for the first block in the vector
) -> Result<Vec<DataBlock>, &'static str> {
	
	let total_written_size = first_block_offset+ data_vector.len();
	if total_written_size / 248 > data_block_vector.len()*248 {
		Err("trying to write outside of blocks")		
	} 
	
	let remaining_data = data_vector.split_off(248-first_block_offset);	
	data_block_vector[0].set_data(data_vector,first_block_offset);
	let remaining_data_iderator = remaining_data.iter();
	for i in 1..data_block_vector.len(){
		let mut data_to_write : Vec<u8>= Vec::New();
		for j in 1..248{
			let current_byte = match remaining_data_iderator.next(){
				None => break,
				Some(byte) => byte
			};
			data_to_write.push(current_byte);
		}
		data_block_vector[i].set_data()
	}
	Ok(data_block_vector)
}











pub fn write_to_file(
	drive_file : &File,
	offset : u64,
	data : Vec<u8>,
	start_block : StartBlock,
	
) -> Result<u32, &'static str>{


	//first need to check if the file size needs to be changed and if it needs more blocks
	let total_size = start_block.get_size();
	let data_size = data.len();
	let final_changed_byte : u32 = offset + data_size;
	let total_inital_block_size = (data_size / 248.0).ceil();
	let final_modified_block_size = (final_changed_byte / 248.0).ceil();
	if total_inital_block_size < final_modified_block_size {
		start_block.set_size(final_changed_byte);
		start_block_write(drive_file,start_block);
		expand_file(
			drive_file,
			start_block,
			final_modified_block_size - total_inital_block_size,
			match start_block.get_file_type(){
				3 => true,
				_ => false,
			}
		)
	}
	let current_full_data_block_vector : Vec<DataBlock> = get_data_block_vec_from_start_block(drive_file,start_block);
	let mut modified_data_block_vector : Vec<DataBlock> = Vec::new(); 
	for i in offset / 248 .. final_changed_byte/248+1{
		modified_data_block_vector.push(current_full_data_block_vector.get(i));
	}
	let first_block_offset = offset % 248;
	
	let blocks_to_be_written = data_block_vector_set_data_from_data_vector(modified_data_block_vector,data,first_block_offset);

	for block in blocks_to_be_written{
		data_block_write(drive_file,block);
	}
	Ok(start_block.get_size())



}










// to write to file you must 
//		block offset and byte offset
//		check if the size of the data requires more blocks to be attached to the file
//		attach the blocks
//		split data to each block
//		write the data to each block
//		update file size






