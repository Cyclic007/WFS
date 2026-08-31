use super::super::blocks::{StartBlock, DataBlock};
use super::super::results::*;
use super::block_actions::*;
use std::{
    fs::File,
    os::unix::fs::FileExt,
	ffi::{OsStr,OsString}
	
};
use fuse_mt::{DirectoryEntry,FileType,ResultReaddir,ResultEmpty};




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

/// gets all of the [DataBlock]s within a file
pub fn get_data_block_vec_from_start_block(
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


/// grabs a vector of data [u8] from all of the data blocks in a vector
pub fn get_data_vec_from_data_block_vec(
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




/// grabs a vector of block indexes [u32] from all of the data blocks in a vector
pub fn get_directory_index_vec_from_data_block_vec(
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




/// grabs a vector of block indexes [u32] from the data attached to a start block
pub fn get_directory_data_from_start_block_index(
	drive_file : &File,
	start_block_index : u32
) -> ResultBlockIndexVector{
	get_directory_index_vec_from_data_block_vec(
		get_data_block_vec_from_start_block_index(drive_file,start_block_index)?
	)
}



/// modifies a data block vector to now hold new data from a data vector 
pub fn data_block_vector_set_data_from_data_vector(
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

/// grabs all data from a file attached to a start block
pub fn get_data_vector_from_start_block(
	drive_file : &File,
	start_block : &mut StartBlock
) -> ResultDataVector {
	Ok(
		get_data_vec_from_data_block_vec(
			get_data_block_vec_from_start_block(
				drive_file,
				start_block.clone()
			)?
		)?
	)
}

/// writes an entire [Vec] of [DataBlock] that don't have correct indexes to storage
pub fn data_block_vector_write(
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
