//This will define each block and the functions therin




// each block is a total length of 256 bytes (2048 bits)
// there is no real diffrence between a file data block and a directory data block
// all blocks are diagramed in the spreadsheet file in the root of the git repo
// block indexes are 4 bytes long (32 bits)
// subsequint data blocks do not need to be consecutive when stored on drive
// a null block index is 4294967295


//block definitions


// holds all metadata
#[derive(Clone)]
pub struct StartBlock{
	block_index 			: u32, 			// 0x00  ->  0x03
	first_data_block_index 	: u32, 			// 0x04  ->  0x07
	perms 					: u16,			// 0x08  ->  0x09
	file_type 				: u16,			// 0x0A  ->  0x0B 
	//4 bytes padding						// 0x0C  ->  0x0F
	a_time 					: u64,			// 0x10  ->  0x17
	m_time 					: u64,			// 0x18  ->  0x1F
	
	user_id 				: u32,			// 0x20  ->  0x23
	group_id 				: u32,			// 0x24  ->  0x27
	size 					: u64,			// 0x28  ->  0x2F

	name 					: [u8 ; 208]	// 0x30  ->  0xFF
	
}

// holds any and all data
#[derive(Clone)]
pub struct DataBlock{
	block_index 			: u32, 			// 0x00  ->  0x03

	data 					: [u8 ; 248],	// 0x04  ->  0xFB

	next_block_index 		: u32			// 0xFC  ->  0xFF
}

#[derive(Clone)]
pub struct RawBlock{
	pub data : [u8 ; 256]					// 0x00  ->  0xFF
}

#[derive(Clone)]
pub struct RawBlockWithIndex{
	pub block_index : u32,					// 0x00  ->  0x03
	pub data : [u8;252]						// 0x04  ->  0xFF
}



impl StartBlock {
	pub fn new(
		block_index : u32, 					// 0x00  ->  0x03
		first_data_block_index : u32, 		// 0x04  ->  0x07
		perms : u16,						// 0x08  ->  0x09
		file_type : u16,					// 0x0A  ->  0x0B 
		//4 bytes padding					// 0x0C  ->  0x0F
		a_time : u64,						// 0x10  ->  0x17
		m_time : u64,						// 0x18  ->  0x1F
		
		user_id : u32,						// 0x20  ->  0x23
		group_id : u32,						// 0x24  ->  0x27
		size : u64,							// 0x28  ->  0x2F
	
		name : [u8 ; 208]					// 0x30  ->  0xFF
	) -> Self
	{
		Self {
			block_index,
			first_data_block_index,
			perms,
			file_type,
			a_time,
			m_time,
			user_id,
			group_id,
			size,
			name
		}
	}




	//getters
	pub fn get_block_index(&self) -> &u32 {&self.block_index}	
	pub fn get_first_data_block_index(&self) -> &u32{&self.first_data_block_index}
	pub fn get_perms(&self) -> &u16{&self.perms}
	pub fn get_file_type(&self) -> &u16{&self.file_type}
	pub	fn get_a_time(&self) -> &u64{&self.a_time}
	pub	fn get_m_time(&self) -> &u64{&self.m_time}
	pub	fn get_user_id(&self) -> &u32{&self.user_id}
	pub	fn get_group_id(&self) -> &u32{&self.group_id}
	pub	fn get_size(&self) -> &u64{&self.size}
	pub fn get_raw_name(&self) -> [u8 ; 208] {self.name.clone()}
	pub fn get_name(&self) -> String {String::from_utf8(self.name.clone().to_vec()).unwrap()}

	//setters
	pub fn set_block_index(&mut self, new_index : u32) {self.block_index = new_index}
	pub fn set_first_data_block_index(&mut self, new_index: u32){self.first_data_block_index = new_index}
	pub fn set_perms(&mut self, new_perms : u16) {self.perms = new_perms}
	pub fn set_file_type(&mut self, new_file_type : u16) {self.file_type = new_file_type}
	pub fn set_a_time(&mut self, new_a_time : u64) {self.a_time = new_a_time}
	pub fn set_m_time(&mut self, new_m_time : u64) {self.m_time = new_m_time}
	pub fn set_user_id(&mut self, new_user_id : u32) {self.user_id = new_user_id}
	pub fn set_group_id(&mut self, new_group_id : u32) {self.group_id = new_group_id}
	pub fn set_size(&mut self, new_size : u64) {self.size = new_size}
	pub fn set_raw_name(&mut self, new_raw_name : [u8 ; 208]) {self.name = new_raw_name}
	pub fn set_name (&mut self, new_name : &str) {self.name = <[u8;208]>::try_from(new_name.as_bytes()).unwrap()}
}



impl DataBlock {

	pub fn new(
		block_index 			: u32, 			// 0x00  ->  0x03
		
		data 					: [u8 ; 248],	// 0x04  ->  0xFB
	
		next_block_index 		: u32			// 0xFC  ->  0xFF
	) -> Self
	{
		Self{
			block_index,
			data,
			next_block_index,
		}
	}

	pub fn new_plain_dir (
		block_index : u32
	) -> Self{
		Self{
			block_index,
			data : [255; 248],
			next_block_index : u32::MAX,
		}
	}
	pub fn new_plain_file (
		block_index : u32
	) -> Self{
		Self{
			block_index,
			data : [0; 248],
			next_block_index : u32::MAX,
		}
	}	
	pub fn new_plain(
		block_index : u32,
		is_directory : bool
	) -> Self {
		match is_directory {
			true => Self::new_plain_dir(block_index),
			false => Self::new_plain_file(block_index),
		}
	}
	

	pub fn get_block_index(&self) -> &u32 {&self.block_index}
	pub fn get_data(&self) -> &[u8 ; 248] {&self.data}
	pub fn get_next_block_index(&self) -> &u32 {&self.next_block_index}

	pub fn set_block_index(&mut self, new_index : u32) {self.block_index = new_index}
	pub fn set_next_block_index(&mut self, new_index : u32) {self.next_block_index = new_index}


	// This is specal and requires an offset so that you dont need to remove the data entirely to change it
	pub fn set_data(&mut self , new_data : Vec<u8>, offset : usize) -> Result<u32, &'static str>{
		if offset + new_data.len() > 248{
			return Err("trying to write outside of block")
		}
	
		for i in 0..new_data.len(){
			self.data[i+offset] = new_data[i]
		}
		Ok(20)
	}



	
}


impl From<RawBlock> for DataBlock{
	fn from(raw_block : RawBlock) -> Self{
		Self{
			block_index : u32::from_ne_bytes(<[u8;4]>::try_from(&raw_block.data[0..4]).unwrap()),
			data : <[u8;248]>::try_from(&raw_block.data[4..252]).unwrap(),
			next_block_index : u32::from_ne_bytes(<[u8;4]>::try_from(&raw_block.data[252..256]).unwrap()),

		}
	}	
}

impl From<RawBlock> for StartBlock{
	fn from(raw_block : RawBlock) -> Self{
		Self{
			block_index : u32::from_ne_bytes(<[u8;4]>::try_from(&raw_block.data[0..4]).unwrap()),
			first_data_block_index  : u32::from_ne_bytes(<[u8;4]>::try_from(&raw_block.data[4..8]).unwrap()),
			perms :  u16::from_ne_bytes(<[u8;2]>::try_from(&raw_block.data[8..10]).unwrap()),
			file_type : u16::from_ne_bytes(<[u8;2]>::try_from(&raw_block.data[10..12]).unwrap()),
			//padding
			a_time :  u64::from_ne_bytes(<[u8;8]>::try_from(&raw_block.data[16..24]).unwrap()),
			m_time :  u64::from_ne_bytes(<[u8;8]>::try_from(&raw_block.data[24..32]).unwrap()),
			user_id : u32::from_ne_bytes(<[u8;4]>::try_from(&raw_block.data[32..36]).unwrap()),
			group_id : u32::from_ne_bytes(<[u8;4]>::try_from(&raw_block.data[36..40]).unwrap()),
			size : u64::from_ne_bytes(<[u8;8]>::try_from(&raw_block.data[40..48]).unwrap()),
			name : <[u8;208]>::try_from(&raw_block.data[48..256]).unwrap()
		}
	}
}





impl From<DataBlock> for RawBlock{
	fn from(data_block : DataBlock) -> Self{
		let mut data_vec : Vec<u8> = Vec::with_capacity(256);

		for byte in data_block.get_block_index().to_ne_bytes(){
			data_vec.push(byte);
		}
		for byte in data_block.data{
			data_vec.push(byte);
		}
		for byte in data_block.get_next_block_index().to_ne_bytes(){
			data_vec.push(byte);
		}
		let mut data_arr : [u8;256] = [0;256];
		for i in 0..256{
			data_arr[i] = data_vec[i];
		}
		Self{
			data : data_arr
		}
	}
}


impl From<StartBlock> for RawBlock{
	fn from(start_block : StartBlock) -> Self{
		let mut data_vec : Vec<u8> = Vec::with_capacity(256);

		for byte in start_block.get_block_index().to_ne_bytes(){
			data_vec.push(byte);
		}
		for byte in start_block.get_first_data_block_index().to_ne_bytes(){
			data_vec.push(byte);
		}		
		for byte in start_block.get_perms().to_ne_bytes(){
			data_vec.push(byte);
		}
		for byte in start_block.get_file_type().to_ne_bytes(){
			data_vec.push(byte);
		}
		for _i in 0..5{
			data_vec.push(0);
		}
		for byte in start_block.get_a_time().to_ne_bytes(){
			data_vec.push(byte);
		}
		for byte in start_block.get_m_time().to_ne_bytes(){
			data_vec.push(byte);
		}
		for byte in start_block.get_user_id().to_ne_bytes(){
			data_vec.push(byte);
		}
		for byte in start_block.get_group_id().to_ne_bytes(){
			data_vec.push(byte);
		}
		for byte in start_block.get_size().to_ne_bytes(){
			data_vec.push(byte);
		}
		for byte in start_block.get_raw_name(){
			data_vec.push(byte);
		}
		
		let mut data_arr : [u8;256] = [0;256];
		for i in 0..256{
			data_arr[i] = data_vec[i];
		}
		Self{
			data : data_arr
		}
	}
}
