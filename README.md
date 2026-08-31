# WFS
Custom FUSE file system 

## The why?

This was made because Posix is an interesting spec and I wanted to make a Filesystem with FUSE to prove that with enough will you can just make things.

## The how 

This is made using the fuse_mt crate (While on a hamiltion the musical bender) and has a lot of the POSIX spec implemented.

## running

To run you use the executible like this

'''
WFS <target> <mount>
'''

when using a new formatted target file you must ensure that the entire target file is full of all zeros

this can be done by running

'''
dd if=/dev/zero of=<drive_file_name.img> count=<size> 
'''

## testing

this filesystem implements rust test cases

### Setup

for the tests to work you need to make a directory at the root of the git repositiory called "test" and then you run "sudo cargo test"

Tests must be run as sudo because it needs to be able to automaticly unmount the filesystem

'''
mkdir test
sudo cargo test
'''

## Implementaion specs

- this uses a static block size of 256 bytes
- each data block can store 248 bytes of data
- the max name size of a file is 208 bytes long
- links do NOT WORK (because of deletion implemenation)
- when a file is deleted all of its blocks are written to 0
- there is no chacheing in this system
- only root can change perms
- only root can change owners
- when directories are deleted ALL files in the directory are deleted


# Windows (EWW)

## THIS FILE SYSTEM DOES IN NO WAY NATIVELY SUPPORT WINDOWS

if you want to try and mount a FUSE file system on windows using WSL you can try but NO support will be given


