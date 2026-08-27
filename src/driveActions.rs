// This will define shortcuts for minipulating the data on the drive
use std::{
    fs::File,
    os::unix::fs::FileExt,
	ffi::{OsStr,OsString}
};
use super::blocks::{StartBlock, DataBlock, RawBlock};
use fuse_mt::{DirectoryEntry,FileType};
use fuse_mt::*;
use super::results::*;
fn get_offset_from_block_index(block_index : u32) -> ResultBlockIndex{
	Ok(u32::try_from(block_index*256).unwrap())
}

fn direct_raw_block_read(
	drive_file : &File,
	block_index : u32,	
) -> ResultRawBlock{
	if block_index == u32::MAX {
		return Err(libc::EIO)
	}
	let read_offset = get_offset_from_block_index(block_index)?;
	let mut read_buffer : [u8 ; 256] = [0 ; 256];
	drive_file.read_exact_at(&mut read_buffer , u64::try_from(read_offset).unwrap());
	Ok(
		RawBlock{
			data : read_buffer
		}
	)

}

fn direct_raw_block_write(
	drive_file : &File,
	block_index : u32,
	block : RawBlock
) -> ResultEmpty{
	if block_index == u32::MAX {
		return Err(libc::EIO)
	}
	let write_offset = get_offset_from_block_index(block_index)?;
	drive_file.write_all_at(&block.data,u64::try_from(write_offset).unwrap());
	Ok(())
}


pub fn data_block_read(
	drive_file : &File,
	block_index: u32,
) -> ResultDataBlock{
	Ok(
		DataBlock::from(direct_raw_block_read(drive_file,block_index)?)
	)
}

pub fn start_block_read(
	drive_file : &File,
	block_index: u32,
) -> ResultStartBlock{
	Ok(
		StartBlock::from(direct_raw_block_read(drive_file,block_index)?)
	)
}

pub fn data_block_write(
	drive_file : &File,
	block : DataBlock,
) -> ResultEmpty{
	direct_raw_block_write(
		drive_file,
		block.get_block_index().clone(),
		RawBlock::from(block)
	)
}
pub fn start_block_write(
	drive_file : &File,
	block : StartBlock,
) ->ResultEmpty{
	direct_raw_block_write(
		drive_file,
		block.get_block_index().clone(),
		RawBlock::from(block)
	)
}


fn get_data_block_vec_from_first_data_block_index(
	drive_file : &File,
	first_block_index : u32,
) -> ResultDataBlockVector{
	let mut block_vec : Vec<DataBlock> = Vec::new(); 
	let first_block = data_block_read(drive_file,first_block_index)?;
	block_vec.push(first_block.clone());
	let mut current_next_index = first_block.clone().get_next_block_index().clone();
	while current_next_index.clone() != u32::MAX{
		let current_block = data_block_read(drive_file, current_next_index)?;
		block_vec.push(current_block.clone());
		current_next_index = current_block.clone().get_next_block_index().clone();
	}
	Ok(block_vec)
}


fn get_data_block_vec_from_start_block(
	drive_file : &File,
	start_block : StartBlock
) -> ResultDataBlockVector {
	get_data_block_vec_from_first_data_block_index(
		drive_file,
		start_block.clone().get_first_data_block_index().clone()
	)
}

fn get_data_block_vec_from_start_block_index(
	drive_file : &File,
	start_block_index : u32,
) -> ResultDataBlockVector{
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
) -> ResultDataVector{
	let mut data_vec : Vec<u8> = Vec::new();
	for block in data_block_vec{
		for byte in block.get_data().clone(){
			data_vec.push(byte)
		}
	}
	Ok(data_vec)
}


fn get_directory_index_vec_from_data_block_vec(
	data_block_vec : Vec<DataBlock>
) -> ResultBlockIndexVector{
	let mut directory_vector : Vec<u32> = Vec::new();
	let data_vec : Vec<u8> = get_data_vec_from_data_block_vec(data_block_vec)?;
	for directory_chunk in data_vec.chunks_exact(4){
		let directory_index_buffer : u32 = u32::from_ne_bytes(<[u8;4]>::try_from(directory_chunk).unwrap());
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
)  -> ResultDataVector{
	get_data_vec_from_data_block_vec(
		get_data_block_vec_from_start_block_index(drive_file,start_block_index)?
	)
}


fn get_directory_data_from_start_block_index(
	drive_file : &File,
	start_block_index : u32
) -> ResultBlockIndexVector{
	get_directory_index_vec_from_data_block_vec(
		get_data_block_vec_from_start_block_index(drive_file,start_block_index)?
	)
}



pub fn get_directory_entry_vec_from_start_block_index(
	drive_file : &File,
	start_block_index : u32
) -> ResultReaddir {
	let directory_indexes = get_directory_data_from_start_block_index( drive_file, start_block_index)?;
	let mut directory_entries : Vec<DirectoryEntry> = Vec::new();
	for index in directory_indexes {
		let current_start_block = start_block_read(drive_file,index)?;
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


fn get_start_block_from_name_and_directory_start_block_index(
	drive_file: &File,
	directory_start_block_index : u32,
	name : OsString,
) -> ResultStartBlockOption  {
	let directory_data = get_directory_data_from_start_block_index(drive_file,directory_start_block_index)?;
	for index in directory_data{
		let current_start_block = start_block_read(drive_file, index)?;
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
	let name_list : Vec<&str> = path.to_str().unwrap().splitn(20, "/").collect();
	let mut current_block_index = 1;
	for name in name_list.clone(){
		let current_start_block = get_start_block_from_name_and_directory_start_block_index ( drive_file , current_block_index , OsString::from(name))?.unwrap();
		if current_start_block.get_name().eq(name_list[name_list.clone().len()-1]){
			return Ok(Some(current_start_block))
		}
		current_block_index = current_start_block.get_block_index().clone();
	}
	return Ok(None)
}

fn find_next_empty_blocks(
	drive_file : &File,
	number_of_blocks : usize,
) -> ResultBlockIndexVector {
	let mut index_vec : Vec<u32> = Vec::with_capacity(number_of_blocks);
	for i in 0..u32::MAX{
		if data_block_read(drive_file,i).unwrap().get_block_index().clone() != i{
			index_vec.push(i);
			if index_vec.len() == number_of_blocks{
				return Ok(index_vec)
			}
		}
	}
	return Ok(index_vec)
}






//This does not add it to a parent directory
pub fn write_new_empty_directory (
	drive_file : &File,
	start_block : &mut StartBlock,
) -> ResultStartBlock{
	let empty_blocks = find_next_empty_blocks(drive_file,2)?;
	start_block.set_block_index(empty_blocks[0]);
	start_block.set_first_data_block_index(empty_blocks[1]);
	data_block_write(drive_file,DataBlock::new_plain_dir(empty_blocks[1]));
	start_block_write(drive_file, start_block.clone());
	return Ok(start_block.clone())
}

pub fn write_new_empty_file (
	drive_file : &File,
	start_block : &mut StartBlock,
) -> ResultStartBlock{
	let empty_blocks = find_next_empty_blocks(drive_file,2)?;
	start_block.set_block_index(empty_blocks[0]);
	start_block.set_first_data_block_index(empty_blocks[1]);
	data_block_write(drive_file,DataBlock::new_plain_file(empty_blocks[1]));
	start_block_write(drive_file, start_block.clone());
	return Ok(start_block.clone())
}




pub fn expand_file (
	drive_file : &File,
	start_block : StartBlock,
	number_of_additonal_blocks : usize,
	is_directory : bool
) -> ResultEmpty {
	let current_data_block_index = start_block.get_first_data_block_index().clone();
	let mut current_data_block = data_block_read(drive_file,current_data_block_index)?;
	let current_data_block_index = current_data_block.get_next_block_index().clone();
	while current_data_block_index != u32::MAX{
		let current_data_block = data_block_read(drive_file,current_data_block_index)?;
		let _current_data_block_index = current_data_block.get_next_block_index();
	}

	let new_block_indexes = find_next_empty_blocks(drive_file,number_of_additonal_blocks)?;
	current_data_block.set_next_block_index(new_block_indexes[0]);
	data_block_write(drive_file,current_data_block);
	for i in 1..new_block_indexes.len(){
		data_block_write(
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
			data_block_write(drive_file,DataBlock::new_plain(new_block_indexes[i],is_directory));
			break;
		}
	}
	Ok(())
}





//Index vector must be the same size as the data block vector
fn data_block_vector_write(
	drive_file : &File,
	mut data_block_vector : Vec<DataBlock>,
	data_block_index_vector : Vec<u32>
) -> ResultEmpty {
	if data_block_index_vector.len() != data_block_index_vector.len(){
		return Err(libc::EDOM)
	}

	
	for i in 0..data_block_index_vector.len(){
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
	Ok(())
	
}







fn data_block_vector_set_data_from_data_vector(
	data_block_vector : &mut Vec<DataBlock>,
	data_vector : &mut Vec<u8>,
	first_block_offset : usize, //Offset to start writeing only for the first block in the vector
	block_offset : usize,
) -> ResultDataBlockVector {
	
	let total_written_size = first_block_offset+ data_vector.len();
	if total_written_size / 248 > data_block_vector.len()*248 {
		return Err(libc::ERANGE)		
	} 
	
	let remaining_data = data_vector.split_off(248-first_block_offset);	
	data_block_vector[block_offset].set_data(data_vector.clone(),first_block_offset);
	let mut remaining_data_iderator = remaining_data.iter();
	for i in 1..data_block_vector.len(){
		let mut data_to_write : Vec<u8> = Vec::new();
		for _j in 1..248{
			let current_byte = match remaining_data_iderator.next(){
				None => break,
				Some(byte) => byte
			};
			data_to_write.push(current_byte.clone());
		}
		data_block_vector[i+block_offset].set_data(data_to_write,0);
	}
	Ok(data_block_vector.clone())
}











pub fn write_to_file(
	drive_file : &File,
	offset : usize,
	data : &mut Vec<u8>,
	start_block : &mut StartBlock,
	
) -> ResultSize{


	//first need to check if the file size needs to be changed and if it needs more blocks
	let _total_size = start_block.get_size();
	let data_size = data.len();
	let final_changed_byte  = offset + data_size;
	let total_inital_block_size = (f64::try_from(u32::try_from(data_size).unwrap()).unwrap() / 248.0).ceil();
	let final_modified_block_size : usize = (f64::try_from(u32::try_from(final_changed_byte).unwrap()).unwrap() / 248.0).ceil() as usize;
	if total_inital_block_size < final_modified_block_size as f64 {
		start_block.set_size(u64::try_from(final_changed_byte).unwrap());
		start_block_write(drive_file,start_block.clone());
		expand_file(
			drive_file,
			start_block.clone(),
			final_modified_block_size - total_inital_block_size as usize,
			match start_block.get_file_type(){
				3 => true,
				_ => false,
			}
		);
	}
	let current_full_data_block_vector : Vec<DataBlock> = get_data_block_vec_from_start_block(drive_file,start_block.clone())?;
	let mut modified_data_block_vector : Vec<DataBlock> = Vec::new(); 
	for i in offset / 248 .. final_changed_byte/248+1{
		modified_data_block_vector.push(current_full_data_block_vector.get(i).unwrap().clone());
	}
	let first_block_offset = offset % 248;
	
	let blocks_to_be_written : Vec<DataBlock> = data_block_vector_set_data_from_data_vector(&mut modified_data_block_vector,data,first_block_offset,0)?;
	
	for block in blocks_to_be_written{
		data_block_write(drive_file,block);
	}
	Ok(usize::try_from(start_block.clone().get_size().clone()).unwrap())



}



pub fn check_if_block_is_empty(
	drive_file : &File,
	block_index_to_be_checked : u32
) -> ResultBool{
	let raw_block = direct_raw_block_read(drive_file,block_index_to_be_checked)?;
	let data = raw_block.data;
	for byte in data{
		if byte != 0 {
			return Ok(false)
		}
	}
	Ok(true)
}


pub fn normal_data_block_vector_write(
	drive_file : &File,
	data_block_vector : Vec<DataBlock>,
) -> ResultEmpty  {
	for block in data_block_vector{
		data_block_write(drive_file,block);
	}
	Ok(())
}




fn modify_directory_contents(
	drive_file : &File,
	directory_entry_index : u32,
	new_directory_entry_data : u32,
	directory_start_block : &mut  StartBlock,
) -> ResultEmpty {
	
	
	let mut data_block_vec = get_data_block_vec_from_start_block(drive_file,directory_start_block.clone())?;
	let new_data = new_directory_entry_data.to_ne_bytes();
	let data_offset : usize = usize::try_from(directory_entry_index*4).unwrap();


	
	let modified_data_block_vec = data_block_vector_set_data_from_data_vector(
		&mut data_block_vec,
		&mut new_data.to_vec(),
		data_offset % 248,
		data_offset / 248
	).unwrap();
	normal_data_block_vector_write(drive_file,modified_data_block_vec);
	Ok(())
}










fn append_directory_content(
	drive_file : &File,
	new_directory_entry_data : u32,
	directory_start_block : &mut StartBlock
) -> ResultEmpty{
	let directory_data = get_directory_data_from_start_block_index(drive_file,directory_start_block.get_block_index().clone())?;
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

	expand_file(drive_file,directory_start_block.clone(),1,true)?;
	let directory_data = get_directory_data_from_start_block_index(drive_file,directory_start_block.get_block_index().clone())?;
	for i in 0..directory_data.len(){
		if directory_data.get(i).unwrap().clone() == u32::MAX{
			modify_directory_contents(drive_file, u32::try_from(i).unwrap(),new_directory_entry_data,directory_start_block)?;
			return Ok(())	
		}
	}
	Ok(())
}





pub fn create_empty_directory(
	drive_file : &File,
	parent_start_block : &mut StartBlock,
	child_start_block : &mut StartBlock
	
) -> ResultBlockIndex {
	let new_child_start_block = write_new_empty_directory(drive_file,&mut child_start_block.clone())?;
	append_directory_content(drive_file,new_child_start_block.get_block_index().clone(),parent_start_block);
	Ok(child_start_block.get_block_index().clone())
}

pub fn create_empty_file(
	drive_file : &File,
	parent_start_block : &mut StartBlock,
	child_start_block : &mut StartBlock
) -> ResultBlockIndex {
	let new_child_start_block = write_new_empty_file(drive_file,&mut child_start_block.clone())?;
	append_directory_content(drive_file,new_child_start_block.get_block_index().clone(),parent_start_block);
	Ok(child_start_block.get_block_index().clone())
}








// to write to file you must 
//		block offset and byte offset
//		check if the size of the data requires more blocks to be attached to the file
//		attach the blocks
//		split data to each block
//		write the data to each block
//		update file size






