// this will define all function results
use super::blocks::*;






pub type ResultDataBlock = Result<DataBlock, libc::c_int>;
pub type ResultStartBlock = Result<StartBlock, libc::c_int>;
pub type ResultDataBlockVector = Result<Vec<DataBlock>, libc::c_int>;
pub type ResultRawBlock = Result<RawBlock, libc::c_int>;
pub type ResultBlockIndex = Result<u32, libc::c_int>;
pub type ResultBlockIndexVector = Result<Vec<u32>, libc::c_int>;
pub type ResultSize = Result<usize, libc::c_int>;
pub type ResultBool = Result<bool, libc::c_int>;
pub type ResultDataVector = Result<Vec<u8>, libc::c_int>;
pub type ResultStartBlockOption = Result<Option<StartBlock>, libc::c_int>;
pub type ResultHandleIndex = Result<u64, libc::c_int>;
