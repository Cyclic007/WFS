use super::super::blocks::{StartBlock, DataBlock, RawBlock};
use super::super::results::*;
use fuse_mt::ResultEmpty;
use std::{
    fs::File,
    os::unix::fs::FileExt,
	ffi::{OsStr,OsString}
};

///this is a quick and dirty math bit
fn get_offset_from_block_index(block_index : u32) -> ResultBlockIndex{
	Ok(block_index*256)
}


/// deletes a single block from storage
pub fn direct_block_deletion(
	drive_file : &File,
	block_index : u32
) -> ResultEmpty {
	if block_index == u32::MAX {
		return Err(libc::EIO)
	}
	let write_offset = get_offset_from_block_index(block_index)?;
	let crap : [u8;256] = [0;256];
	drive_file.write_all_at(&crap,u64::from(write_offset)).unwrap();
	Ok(())
}

/// This will read data from storage 
fn direct_raw_block_read(
	drive_file : &File,
	block_index : u32,	
) -> ResultRawBlock{
	if block_index == u32::MAX {
		return Err(libc::EIO)
	}
	let read_offset = get_offset_from_block_index(block_index)?;
	let mut read_buffer : [u8 ; 256] = [0 ; 256];
	drive_file.read_exact_at(&mut read_buffer , u64::from(read_offset)).unwrap();
	Ok(
		RawBlock{
			data : read_buffer
		}
	)

}


/// writes 256 bytes of data to storage
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



/// this will read a block from storage and cast it to a data block [DataBlock]
/// this does not check if the block is actualy supposed to be holding data
pub fn data_block_read(
	drive_file : &File,
	block_index: u32,
) -> ResultDataBlock{
	Ok(
		DataBlock::from(direct_raw_block_read(drive_file,block_index)?)
	)
}


/// this will read a block from storage and cast it to a start block [StartBlock]
/// this does not check if the block is actualy supposed to be holding metadata
pub fn start_block_read(
	drive_file : &File,
	block_index: u32,
) -> ResultStartBlock{
	Ok(
		StartBlock::from(direct_raw_block_read(drive_file,block_index)?)
	)
}


/// this will write a data block [DataBlock] to storage
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
/// this will write a start block [StartBlock] to storage
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


/// This writes all of the blocks in a vector to storage
pub fn bulk_data_block_write(
	drive_file : &File,
	data_block_vector : Vec<DataBlock>
) -> ResultEmpty{
	for block in data_block_vector{
		data_block_write(drive_file,block)?;
	}
	Ok(())
}


/// this checks if a block contains the right index for the location of the block
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
