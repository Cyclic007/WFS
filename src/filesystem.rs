// import all of the things 

use fuse_mt::*;
use std::path::{Path,PathBuf};
use std::fs::*;
use std::time::SystemTime;
use std::ffi::{ OsStr, OsString};

use super::blocks::{StartBlock};
use super::drive_actions::*;
use super::handles::*;

/// The Filesystem Implemetaion [FilesystemMT]
pub struct WeirdFileSystem{
	target : OsString,
}


impl WeirdFileSystem{
	pub fn new(os : OsString) -> Self{
		Self{
			target : os,
		}
	}
}




impl FilesystemMT for WeirdFileSystem{
    fn init(&self, req: RequestInfo) -> ResultEmpty {
		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();
		if block_actions::check_if_block_is_empty(&root,1).expect("could not check if block is empty"){

			let mut new_start_block = StartBlock::new(
				0,
				1,
				551, // 777 in octal
				3,
				SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("time is broken").as_secs(),
				SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("time is broken").as_secs(),
				req.uid,
				req.gid,
				0,
				[0;208]
			);
			write_utils::write_new_empty_directory(
				&root,
				&mut new_start_block
			)?;
		}
		


        Ok(())
    }

    /// Called on filesystem unmount.
    fn destroy(&self) {
        // Nothing.
    }

    /// Get the attributes of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    fn getattr(&self, req: RequestInfo, path: &Path, fh: Option<u64>) -> ResultEntry {
		self.access(req,path,6)?;
        let current_time = SystemTime::now();

        let root = File::options()
        		    .read(true)
        		    .write(true)
        		    .open(self.target.clone()).unwrap();
        let mut handle = match fh {
        	Some(num) => FileHandle::read(num),
        	None => FileHandle::new(Box::from(path))
        };

        let start_block = block_actions::start_block_read(&root,handle.get_start_block_index(&root)?)?;
        let file_attr = start_block.get_file_attr();
        let ttl = current_time.elapsed().unwrap();

		Ok((ttl,file_attr))
    }

    // The following operations in the FUSE C API are all one kernel call: setattr
    // We split them out to match the C API's behavior.

    /// Change the mode of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `mode`: the mode to change the file to.
    fn chmod(&self, req: RequestInfo, path: &Path, fh: Option<u64>, mode: u32) -> ResultEmpty {
       	if req.uid != 0 { //This means the the user is root
       		return Err(libc::EPERM)
       	}
        let root = File::options()
        		    .read(true)
        		    .write(true)
        		    .open(self.target.clone()).unwrap();

        let mut handle = match fh {
        	Some(num) => FileHandle::read(num),
        	None => FileHandle::new(Box::from(path))
        };

        let mut start_block = block_actions::start_block_read(&root,handle.get_start_block_index(&root)?)?;

		start_block.set_perms(u16::try_from(mode).unwrap());
		block_actions::start_block_write(&root,start_block)?;

        Ok(())
    }

    /// Change the owner UID and/or group GID of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `uid`: user ID to change the file's owner to. If `None`, leave the UID unchanged.
    /// * `gid`: group ID to change the file's group to. If `None`, leave the GID unchanged.
    fn chown(&self, req: RequestInfo, path: &Path, fh: Option<u64>, uid: Option<u32>, gid: Option<u32>) -> ResultEmpty {
       	if req.uid != 0 { //This means the the user is root
       		return Err(libc::EPERM)
       	}
		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();

		let mut handle = match fh {
			Some(num) => FileHandle::read(num),
			None => FileHandle::new(Box::from(path))
		};

		let mut start_block = block_actions::start_block_read(&root,handle.get_start_block_index(&root)?)?;

		match uid {
			Some(user_id) => start_block.set_user_id(user_id),
			None => println!("no uid provided")
		}
		match gid {
			Some(group_id) => start_block.set_user_id(group_id),
			None => println!("no gid provided")
		}

		block_actions::start_block_write(&root,start_block)?;
		Ok(())
        
    }

    /// Set the length of a file.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `size`: size in bytes to set as the file's length.
    fn truncate(&self, req: RequestInfo, path: &Path, fh: Option<u64>, size: u64) -> ResultEmpty {
		self.access(req,path,6)?;
		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();

		let mut handle = match fh {
			Some(num) => FileHandle::read(num),
			None => FileHandle::new(Box::from(path))
		};
		let mut start_block = block_actions::start_block_read(&root,handle.get_start_block_index(&root)?)?;
		let current_file_size = start_block.get_size().clone();
		if current_file_size > size {
			file_utils::reduce_file_size(&root, &mut start_block,size)?;
		}
		if current_file_size < size {
			file_utils::expand_file(&root,start_block.clone(),usize::try_from(current_file_size-size).unwrap(),false)?;
		}
		start_block.set_size(size);
		block_actions::start_block_write(&root,start_block)?;
		Ok(())
    }

    /// Set timestamps of a filesystem entry.
    ///
    /// * `fh`: a file handle if this is called on an open file.
    /// * `atime`: the time of last access.
    /// * `mtime`: the time of last modification.
    fn utimens(&self, req: RequestInfo, path: &Path, fh: Option<u64>, atime: Option<SystemTime>, mtime: Option<SystemTime>) -> ResultEmpty {
		self.access(req,path,6)?;
		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();
		let mut handle = match fh {
			Some(num) => FileHandle::read(num),
			None => FileHandle::new(Box::from(path))
		};
		let mut start_block = block_actions::start_block_read(&root,handle.get_start_block_index(&root)?)?;
		match atime {
			Some(time) => start_block.set_a_time(time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
			None => println!("no atime provided")
		}
		match mtime {
			Some(time) => start_block.set_m_time(time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
			None => println!("no mtime provided")
		}

		block_actions::start_block_write(&root,start_block)?;
		Ok(())
    }



    // END OF SETATTR FUNCTIONS

    /// Read a symbolic link.
    fn readlink(&self, _req: RequestInfo, _path: &Path) -> ResultData {
        Err(libc::ENOSYS)
    }

    /// Create a special file.
    ///
    /// * `parent`: path to the directory to make the entry under.
    /// * `name`: name of the entry.
    /// * `mode`: mode for the new entry.
    /// * `rdev`: if mode has the bits `S_IFCHR` or `S_IFBLK` set, this is the major and minor numbers for the device file. Otherwise it should be ignored.
    fn mknod(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _mode: u32, _rdev: u32) -> ResultEntry {
        Err(libc::ENOSYS)
    }

    /// Create a directory.
    ///
    /// * `parent`: path to the directory to make the directory under.
    /// * `name`: name of the directory.
    /// * `mode`: permissions for the new directory.
    fn mkdir(&self, req: RequestInfo, parent: &Path, name: &OsStr, mode: u32) -> ResultEntry {
        self.access(req,parent,6)?;
        let current_time = SystemTime::now();
        let root = File::options()
        		    .read(true)
        		    .write(true)
        		    .open(self.target.clone()).unwrap();
        let mut temp_parent_handle = FileHandle::new(Box::from(parent));
        let parent_start_block_index = temp_parent_handle.get_start_block_index(&root)?;
        let parent_start_block = block_actions::start_block_read(&root,parent_start_block_index)?;
        let mut name_byte_vector : Vec<u8> = Vec::new();

        for byte in name.as_encoded_bytes(){
        	name_byte_vector.push(byte.clone())
        }
        for _i in name.len()..208{
        	name_byte_vector.push(0)
        }

        
        let mut child_start_block = StartBlock::new(
        	0,
        	0,
        	u16::try_from(mode).unwrap(),
        	3,
        	SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        	SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        	req.uid,
        	req.gid,
        	0,
        	<[u8;208]>::try_from(name_byte_vector).unwrap()
        	

        );
        	
        
        write_utils::create_empty_directory(&root,&mut parent_start_block.clone(),&mut child_start_block)?;
        let file_attr = child_start_block.clone().get_file_attr();
        let ttl = current_time.elapsed().unwrap();
        Ok(
        	(
        		ttl,
        		file_attr 
        	)
        )
    }

    /// Remove a file.
    ///
    /// * `parent`: path to the directory containing the file to delete.
    /// * `name`: name of the file to delete.
    fn unlink(&self, req: RequestInfo, parent: &Path, name: &OsStr) -> ResultEmpty {
		self.access(req,parent,6)?;
		self.access(req,&parent.join(name),6)?;

		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();
		let full_path = parent.join(name);
		let mut to_del_handle = FileHandle::new(Box::from(full_path));
		let mut to_del_start_block = block_actions::start_block_read(&root, to_del_handle.get_start_block_index(&root)?)?;
		let mut parent_handle = FileHandle::new(Box::from(parent));
		let mut parent_start_block = block_actions::start_block_read(&root, parent_handle.get_start_block_index(&root)?)?;
		deletion_utils::delete_file(&root,&mut to_del_start_block,&mut parent_start_block)?;
		Ok(())
    }

    /// Remove a directory.
    ///
    /// * `parent`: path to the directory containing the directory to delete.
    /// * `name`: name of the directory to delete.
    fn rmdir(&self, req: RequestInfo, parent: &Path, name: &OsStr) -> ResultEmpty {
		self.access(req,parent,6)?;
		self.access(req,&parent.join(name),6)?;

		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();
		let full_path = parent.join(name);
		let mut to_del_handle = FileHandle::new(Box::from(full_path));
		let mut to_del_start_block = block_actions::start_block_read(&root, to_del_handle.get_start_block_index(&root)?)?;
		let mut parent_handle = FileHandle::new(Box::from(parent));
		let mut parent_start_block = block_actions::start_block_read(&root, parent_handle.get_start_block_index(&root)?)?; 
		deletion_utils::delete_directory(&root, &mut to_del_start_block, &mut parent_start_block)?;   
		Ok(())    
    }

    /// Create a symbolic link.
    ///
    /// * `parent`: path to the directory to make the link in.
    /// * `name`: name of the symbolic link.
    /// * `target`: path (may be relative or absolute) to the target of the link.
    fn symlink(&self, _req: RequestInfo, _parent: &Path, _name: &OsStr, _target: &Path) -> ResultEntry {
        Err(libc::ENOSYS)
    }

    /// Rename a filesystem entry.
    ///
    /// * `parent`: path to the directory containing the existing entry.
    /// * `name`: name of the existing entry.
    /// * `newparent`: path to the directory it should be renamed into (may be the same as `parent`).
    /// * `newname`: name of the new entry.
    fn rename(&self, req: RequestInfo, parent: &Path, name: &OsStr, newparent: &Path, newname: &OsStr) -> ResultEmpty {
		self.access(req,parent,6)?;
		self.access(req,&parent.join(name),6)?;
		self.access(req,newparent,6)?;
		
		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();

		let old_path = parent.join(name);

		let mut old_file_handle = FileHandle::new(Box::from(old_path));
		let mut old_directory_handle = FileHandle::new(Box::from(parent));
		let mut new_directory_handle = FileHandle::new(Box::from(newparent));

		
		let mut file_start_block = block_actions::start_block_read(&root, old_file_handle.get_start_block_index(&root)?)?;
		let mut old_directory_start_block = block_actions::start_block_read(&root, old_directory_handle.get_start_block_index(&root)?)?;
		let mut new_directory_start_block = block_actions::start_block_read(&root, new_directory_handle.get_start_block_index(&root)?)?;

		directory_actions::remove_directory_entry(&root, file_start_block.get_block_index().clone(),&mut old_directory_start_block)?;
		directory_actions::append_directory_content(&root, file_start_block.get_block_index().clone(),&mut new_directory_start_block)?;

		file_start_block.set_name(newname.to_str().unwrap());
		block_actions::start_block_write(&root, file_start_block)?;
		Ok(())
		

		
		
    }

    /// Create a hard link.
    ///
    /// * `path`: path to an existing file.
    /// * `newparent`: path to the directory for the new link.
    /// * `newname`: name for the new link.
    fn link(&self, _req: RequestInfo, _path: &Path, _newparent: &Path, _newname: &OsStr) -> ResultEntry {
        Err(libc::ENOSYS)
    }

    /// Open a file.
    ///
    /// * `path`: path to the file.
    /// * `flags`: one of `O_RDONLY`, `O_WRONLY`, or `O_RDWR`, plus maybe additional flags.
    ///
    /// Return a tuple of (file handle, flags). The file handle will be passed to any subsequent
    /// calls that operate on the file, and can be any value you choose, though it should allow
    /// your filesystem to identify the file opened even without any path info.
    fn open(&self, _req: RequestInfo, path: &Path, flags: u32) -> ResultOpen {
		let root = File::options()
        		    .read(true)
        		    .write(true)
        		    .open(self.target.clone()).unwrap();

        Ok((FileHandle::new(Box::from(path)).allocate_with_index(root)?,flags))
    }

    /// Read from a file.
    ///
    /// Note that it is not an error for this call to request to read past the end of the file, and
    /// you should only return data up to the end of the file (i.e. the number of bytes returned
    /// will be fewer than requested; possibly even zero). Do not extend the file in this case.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `offset`: offset into the file to start reading.
    /// * `size`: number of bytes to read.
    /// * `callback`: a callback that must be invoked to return the result of the operation: either
    ///   the result data as a slice, or an error code.
    ///
    /// Return the return value from the `callback` function.
    fn read(&self, req: RequestInfo, path: &Path, fh: u64, offset: u64, size: u32, callback: impl FnOnce(ResultSlice<'_>) -> CallbackResult) -> CallbackResult {
		match self.access(req,path,4){
			Ok(()) => println!("you have access"),
			Err(error) => return callback(Err(error))
		}

		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();

		let handle = FileHandle::read(fh);
		let mut start_block = block_actions::start_block_read(&root,handle.start_block_index).unwrap();
		
		let	requested_data = file_utils::read_file(
			&root,
			&mut start_block,
			usize::try_from(offset).unwrap(),
			usize::try_from(size).unwrap()
		).expect("could not get data");

        callback(Ok(requested_data.as_slice()))
    }

    /// Write to a file.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `offset`: offset into the file to start writing.
    /// * `data`: the data to write
    /// * `flags`:
    ///
    /// Return the number of bytes written.
    fn write(&self, req: RequestInfo, path: &Path, fh: u64, offset: u64, data: Vec<u8>, _flags: u32) -> ResultWrite {
        self.access(req,path,6)?;
        let root = File::options()
        		    .read(true)
        		    .write(true)
        		    .open(self.target.clone()).unwrap();
		let handle = FileHandle::read(fh);
		let mut start_block = block_actions::start_block_read(&root,handle.start_block_index).unwrap();
		let mut temp_data = data.clone();
		Ok(
			u32::try_from(
				write_utils::write_to_file(
					&root,
					usize::try_from(
						offset
					).unwrap(),
					&mut temp_data,
					&mut start_block
				)?
			).unwrap()
		)
		
        
	}

    /// Called each time a program calls `close` on an open file.
    ///
    /// Note that because file descriptors can be duplicated (by `dup`, `dup2`, `fork`) this may be
    /// called multiple times for a given file handle. The main use of this function is if the
    /// filesystem would like to return an error to the `close` call. Note that most programs
    /// ignore the return value of `close`, though.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `lock_owner`: if the filesystem supports locking (`setlk`, `getlk`), remove all locks
    ///   belonging to this lock owner.
    fn flush(&self, _req: RequestInfo, _path: &Path, _fh: u64, _lock_owner: u64) -> ResultEmpty {
        Ok(())
    }

    /// Called when an open file is closed.
    ///
    /// There will be one of these for each `open` call. After `release`, no more calls will be
    /// made with the given file handle.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `flags`: the flags passed when the file was opened.
    /// * `lock_owner`: if the filesystem supports locking (`setlk`, `getlk`), remove all locks
    ///   belonging to this lock owner.
    /// * `flush`: whether pending data must be flushed or not.
    fn release(&self, _req: RequestInfo, _path: &Path, fh: u64, _flags: u32, _lock_owner: u64, _flush: bool) -> ResultEmpty {
        FileHandle::drop_handle(fh);
        Ok(())
    }

    /// Write out any pending changes of a file.
    ///
    /// When this returns, data should be written to persistent storage.
    ///
    /// * `path`: path to the file.
    /// * `fh`: file handle returned from the `open` call.
    /// * `datasync`: if `false`, also write metadata, otherwise just write file data.
    fn fsync(&self, _req: RequestInfo, _path: &Path, _fh: u64, _datasync: bool) -> ResultEmpty {
        Ok(())
    }

    /// Open a directory.
    ///
    /// Analogous to the `opend` call.
    ///
    /// * `path`: path to the directory.
    /// * `flags`: file access flags. Will contain `O_DIRECTORY` at least.
    ///
    /// Return a tuple of (file handle, flags). The file handle will be passed to any subsequent
    /// calls that operate on the directory, and can be any value you choose, though it should
    /// allow your filesystem to identify the directory opened even without any path info.
    fn opendir(&self, _req: RequestInfo, path: &Path, flags: u32) -> ResultOpen {
		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();

        Ok((FileHandle::new(Box::from(path)).allocate_with_index(root)?,flags))
    }

    /// Get the entries of a directory.
    ///
    /// * `path`: path to the directory.
    /// * `fh`: file handle returned from the `opendir` call.
    ///
    /// Return all the entries of the directory.
    fn readdir(&self, req: RequestInfo, path: &Path, fh: u64) -> ResultReaddir {

		self.access(req,path,4)?;
				
		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();

		let handle = FileHandle::read(fh);
		let start_block = block_actions::start_block_read(&root,handle.start_block_index).unwrap();
		let directory_entries = directory_actions::get_directory_entry_vec_from_start_block_index(&root,start_block.get_block_index().clone())?;
		Ok(directory_entries)
    }

    /// Close an open directory.
    ///
    /// This will be called exactly once for each `opendir` call.
    ///
    /// * `path`: path to the directory.
    /// * `fh`: file handle returned from the `opendir` call.
    /// * `flags`: the file access flags passed to the `opendir` call.
    fn releasedir(&self, _req: RequestInfo, _path: &Path, fh: u64, _flags: u32) -> ResultEmpty {
        FileHandle::drop_handle(fh);
        Ok(())
    }

    /// Write out any pending changes to a directory.
    ///
    /// Analogous to the `fsync` call.
    fn fsyncdir(&self, _req: RequestInfo, _path: &Path, _fh: u64, _datasync: bool) -> ResultEmpty {
        Ok(())
    }

    /// Get filesystem statistics.
    ///
    /// * `path`: path to some folder in the filesystem.
    ///
    /// See the `Statfs` struct for more details.
    fn statfs(&self, req: RequestInfo, path: &Path) -> ResultStatfs {
		self.access(req,path,4)?;
				
		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();

		let handle = FileHandle::new(Box::from(path));
		let start_block = block_actions::start_block_read(&root,handle.start_block_index).unwrap();
		let directory_entries = directory_actions::get_directory_entry_vec_from_start_block_index(&root,start_block.get_block_index().clone())?;
        Ok(
	        Statfs{
	        	blocks : u64::MAX/2,
	        	bfree : u64::MAX/2,
	        	ffree : u64::MAX/4,
	        	bavail : u64::MAX/4,
	        	files : u64::try_from(directory_entries.len()).unwrap(),
	        	bsize : u32::MAX,
	        	namelen : 208,
	        	frsize : 248,
	        }
        )
    }

    /// Set a file extended attribute.
    ///
    /// * `path`: path to the file.
    /// * `name`: attribute name.
    /// * `value`: the data to set the value to.
    /// * `flags`: can be either `XATTR_CREATE` or `XATTR_REPLACE`.
    /// * `position`: offset into the attribute value to write data.
    fn setxattr(&self, _req: RequestInfo, _path: &Path, _name: &OsStr, _value: &[u8], _flags: u32, _position: u32) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Get a file extended attribute.
    ///
    /// * `path`: path to the file
    /// * `name`: attribute name.
    /// * `size`: the maximum number of bytes to read.
    ///
    /// If `size` is 0, return `Xattr::Size(n)` where `n` is the size of the attribute data.
    /// Otherwise, return `Xattr::Data(data)` with the requested data.
    fn getxattr(&self, _req: RequestInfo, _path: &Path, _name: &OsStr, _size: u32) -> ResultXattr {
        Err(libc::ENOSYS)
    }

    /// List extended attributes for a file.
    ///
    /// * `path`: path to the file.
    /// * `size`: maximum number of bytes to return.
    ///
    /// If `size` is 0, return `Xattr::Size(n)` where `n` is the size required for the list of
    /// attribute names.
    /// Otherwise, return `Xattr::Data(data)` where `data` is all the null-terminated attribute
    /// names.
    fn listxattr(&self, _req: RequestInfo, _path: &Path, _size: u32) -> ResultXattr {
        Err(libc::ENOSYS)
    }

    /// Remove an extended attribute for a file.
    ///
    /// * `path`: path to the file.
    /// * `name`: name of the attribute to remove.
    fn removexattr(&self, _req: RequestInfo, _path: &Path, _name: &OsStr) -> ResultEmpty {
        Err(libc::ENOSYS)
    }

    /// Check for access to a file.
    ///
    /// * `path`: path to the file.
    /// * `mask`: mode bits to check for access to.
    ///
    /// Return `Ok(())` if all requested permissions are allowed, otherwise return `Err(EACCES)`
    /// or other error code as appropriate (e.g. `ENOENT` if the file doesn't exist).
    fn access(&self, req: RequestInfo, path: &Path, mask: u32) -> ResultEmpty {
		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();
        let mut temp_handle = FileHandle::new(Box::from(path));
		let start_block_index = temp_handle.get_start_block_index(&root)?;
		let start_block = block_actions::start_block_read(&root, start_block_index)?;
       	if req.uid == 0 { //This means the the user is root
       		return Ok(())
       	}

		// this will determine the exponent to multipy the mask to
		let mut user_relation = 0;
		if req.gid == start_block.get_group_id().clone(){
			user_relation = 1
		}
		if req.uid == start_block.get_user_id().clone(){
			user_relation = 2;
		}
		let base : u16 = 8;

		let bitwise_mask : u16 = u16::try_from(mask).unwrap() * (base.pow(user_relation));
		let bitwise_check = bitwise_mask & start_block.get_perms().clone();
		if bitwise_mask == bitwise_check {
			return Ok(())
		}else {
			return Err(libc::EACCES)
		}
    }

    /// Create and open a new file.
    ///
    /// * `parent`: path to the directory to create the file in.
    /// * `name`: name of the file to be created.
    /// * `mode`: the mode to set on the new file.
    /// * `flags`: flags like would be passed to `open`.
    ///
    /// Return a `CreatedEntry` (which contains the new file's attributes as well as a file handle
    /// -- see documentation on `open` for more info on that).
    fn create(&self, req: RequestInfo, parent: &Path, name: &OsStr, mode: u32, flags: u32) -> ResultCreate {
		self.access(req,parent,6)?;
		let current_time = SystemTime::now();
		let root = File::options()
				    .read(true)
				    .write(true)
				    .open(self.target.clone()).unwrap();
		let mut temp_parent_handle = FileHandle::new(Box::from(parent));
		let parent_start_block_index = temp_parent_handle.get_start_block_index(&root)?;
		let parent_start_block = block_actions::start_block_read(&root,parent_start_block_index)?;
		let mut name_byte_vector : Vec<u8> = Vec::new();

		for byte in name.as_encoded_bytes(){
			name_byte_vector.push(byte.clone())
		}
		for _i in name.len()-1..208{
			name_byte_vector.push(0)
		}

		
		let mut child_start_block = StartBlock::new(
			0,
			0,
			u16::try_from(mode).unwrap(),
			4,
			SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
			SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
			req.uid,
			req.gid,
			30,
			<[u8;208]>::try_from(name_byte_vector).unwrap()
			

		);
			
		
        write_utils::create_empty_file(&root,&mut parent_start_block.clone(),&mut child_start_block)?;
        let child_handle = FileHandle::new(Box::from(parent.join(PathBuf::from(name))));
        let handle_number = child_handle.allocate();
		let file_attr = child_start_block.clone().get_file_attr();
		let ttl = current_time.elapsed().unwrap();
		Ok(
			CreatedEntry{
				fh : handle_number,
				ttl,
				attr: file_attr ,
				flags,
			}
		)
    }
	
}


#[cfg(test)]
mod tests {
	use super::*;
	use sys_mount::*;
	use std::fs;
	#[test]
	fn test_init() {
		fs::write("test/drive_file.img",[0;2000]);
		let data_file_path : OsString = "test/drive_file.img".into();
		let mount_target : OsString= "test/inside".into();
	    let result = unmount("test/inside", UnmountFlags::DETACH);
	    let filesystem =  WeirdFileSystem::new(data_file_path.clone());
	    let fuse_args = [OsStr::new("-o"), OsStr::new("fsname=WFS",)];

	    fuse_mt::spawn_mount(fuse_mt::FuseMT::new(filesystem, 1), &mount_target, &fuse_args[..]);
		let read_result = read_dir("test/inside").unwrap();				
	    let result = unmount("test/inside", UnmountFlags::DETACH);
		
	}

	#[test]
	fn test_file_read_write_small(){
		fs::write("test/drive_file.img",[0;2000]);
		let data_file_path : OsString = "test/drive_file.img".into();
		let mount_target : OsString= "test/inside".into();
	    unmount("test/inside", UnmountFlags::DETACH);
	    let filesystem =  WeirdFileSystem::new(data_file_path.clone());
	    let fuse_args = [OsStr::new("-o"), OsStr::new("fsname=WFS",)];
	    fuse_mt::spawn_mount(fuse_mt::FuseMT::new(filesystem, 1), &mount_target, &fuse_args[..]);
	    
		let zero : u8 = 1;
		fs::write("test/inside/small_test.txt",[zero;200]);
		assert_eq!(fs::read("test/inside/small_test.txt").unwrap(),Vec::from([zero;200]));
		
	   	unmount("test/inside", UnmountFlags::DETACH);
	}
	#[test]
	fn test_file_read_write_big(){
		fs::write("test/drive_file.img",[0;2000]);
		let data_file_path : OsString = "test/drive_file.img".into();
		let mount_target : OsString= "test/inside".into();
	    unmount("test/inside", UnmountFlags::DETACH);
	    let filesystem =  WeirdFileSystem::new(data_file_path.clone());
	    let fuse_args = [OsStr::new("-o"), OsStr::new("fsname=WFS",)];

	    fuse_mt::spawn_mount(fuse_mt::FuseMT::new(filesystem, 1), &mount_target, &fuse_args[..]);
		let zero : u8 = 1;
		fs::write("test/inside/big_test.txt",[zero;400]);
		println!("{}",String::from_utf8(fs::read("test/inside/big_test.txt").unwrap()).unwrap());
		assert_eq!(fs::read("test/inside/big_test.txt").unwrap(),Vec::from([zero;400]));
		
		unmount("test/inside", UnmountFlags::DETACH);
	}

	#[test]
	fn test_mkdir(){
		fs::write("test/drive_file.img",[0;2000]);
		let data_file_path : OsString = "test/drive_file.img".into();
		let mount_target : OsString= "test/inside".into();
	    unmount("test/inside", UnmountFlags::DETACH);
	    let filesystem =  WeirdFileSystem::new(data_file_path.clone());
	    let fuse_args = [OsStr::new("-o"), OsStr::new("fsname=WFS",)];

	    fuse_mt::spawn_mount(fuse_mt::FuseMT::new(filesystem, 1), &mount_target, &fuse_args[..]);
		fs::create_dir("test/inside/test");
		unmount("test/inside", UnmountFlags::DETACH);		
	}
	#[test]
	fn test_small_file_read_write_inside_dir(){
		fs::write("test/drive_file.img",[0;2000]);
		let data_file_path : OsString = "test/drive_file.img".into();
		let mount_target : OsString= "test/inside".into();
	    unmount("test/inside", UnmountFlags::DETACH);
	    let filesystem =  WeirdFileSystem::new(data_file_path.clone());
	    let fuse_args = [OsStr::new("-o"), OsStr::new("fsname=WFS",)];

	    fuse_mt::spawn_mount(fuse_mt::FuseMT::new(filesystem, 1), &mount_target, &fuse_args[..]);
		fs::create_dir("test/inside/test");
		let zero : u8 = 1;
		fs::write("test/inside/test/small_test.txt",[zero;200]);
		println!("{}",String::from_utf8(fs::read("test/inside/test/small_test.txt").unwrap()).unwrap());
		assert_eq!(fs::read("test/inside/test/small_test.txt").unwrap(),Vec::from([zero;200]));
		unmount("test/inside", UnmountFlags::DETACH);				
	}
	#[test]
	fn test_big_file_read_write_inside_dir(){
		fs::write("test/drive_file.img",[0;2000]);
		let data_file_path : OsString = "test/drive_file.img".into();
		let mount_target : OsString= "test/inside".into();
	    unmount("test/inside", UnmountFlags::DETACH);
	    let filesystem =  WeirdFileSystem::new(data_file_path.clone());
	    let fuse_args = [OsStr::new("-o"), OsStr::new("fsname=WFS",)];

	    fuse_mt::spawn_mount(fuse_mt::FuseMT::new(filesystem, 1), &mount_target, &fuse_args[..]);
		fs::create_dir("test/inside/test");
		let zero : u8 = 1;
		fs::write("test/inside/test/big_test.txt",[zero;400]);
		println!("{}",String::from_utf8(fs::read("test/inside/test/big_test.txt").unwrap()).unwrap());
		assert_eq!(fs::read("test/inside/test/big_test.txt").unwrap(),Vec::from([zero;400]));
		unmount("test/inside", UnmountFlags::DETACH);				
	}
	#[test]
	fn test_delete_file(){
		fs::write("test/drive_file.img",[0;2000]);
		let data_file_path : OsString = "test/drive_file.img".into();
		let mount_target : OsString= "test/inside".into();
	    unmount("test/inside", UnmountFlags::DETACH);
	    let filesystem =  WeirdFileSystem::new(data_file_path.clone());
	    let fuse_args = [OsStr::new("-o"), OsStr::new("fsname=WFS",)];
	    fuse_mt::spawn_mount(fuse_mt::FuseMT::new(filesystem, 1), &mount_target, &fuse_args[..]);
	    
		let zero : u8 = 1;
		fs::write("test/inside/del_test.txt",[zero;400]);
		fs::remove_file("test/inside/del_test.txt");
		assert!(!fs::exists("test/inside/del_test.txt").expect("Can't check existence of file del_test.txt"));
		
	    		
	}
	#[test]
	fn test_metadata(){
		fs::write("test/drive_file.img",[0;2000]);
		let data_file_path : OsString = "test/drive_file.img".into();
		let mount_target : OsString= "test/inside".into();
	    unmount("test/inside", UnmountFlags::DETACH);
	    let filesystem =  WeirdFileSystem::new(data_file_path.clone());
	    let fuse_args = [OsStr::new("-o"), OsStr::new("fsname=WFS",)];
	    fuse_mt::spawn_mount(fuse_mt::FuseMT::new(filesystem, 1), &mount_target, &fuse_args[..]);
		let zero : u8 = 1;
		fs::write("test/inside/metadata_test.txt",[zero;400]);
		let meta_data = fs::metadata("test/inside/metadata_test.txt").unwrap();
		assert!(meta_data.is_file());
		assert_eq!(meta_data.len(),400);
	    
	}	
	#[test]
	fn test_rename(){
		fs::write("test/drive_file.img",[0;2000]);
		let data_file_path : OsString = "test/drive_file.img".into();
		let mount_target : OsString= "test/inside".into();
	    unmount("test/inside", UnmountFlags::DETACH);
	    let filesystem =  WeirdFileSystem::new(data_file_path.clone());
	    let fuse_args = [OsStr::new("-o"), OsStr::new("fsname=WFS",)];
	    fuse_mt::spawn_mount(fuse_mt::FuseMT::new(filesystem, 1), &mount_target, &fuse_args[..]);


		fs::create_dir("test/inside/test");	    
		let zero : u8 = 1;
		fs::write("test/inside/rename_test.txt",[zero;400]);
		fs::rename("test/inside/rename_test.txt","test/inside/test/renamed_already_test.txt");
		assert!(!fs::exists("test/inside/rename_test.txt").expect("Can't check existence of file rename_test.txt"));
		assert!(fs::exists("test/inside/test/renamed_already_test.txt").expect("Can't check existence of file /test/renamed_already_test.txt"));
		assert_eq!(fs::read("test/inside/test/renamed_already_test.txt").unwrap(),Vec::from([zero;400]));			
	}


}

