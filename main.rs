#![cfg_attr(not(test), no_std)]

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FileSystemResult<T: Copy + Clone> {
    Ok(T),
    Err(FileSystemError),
}

impl<T: Copy + Clone> FileSystemResult<T> {
    pub fn unwrap(&self) -> T {
        match self {
            FileSystemResult::Ok(v) => *v,
            FileSystemResult::Err(e) => panic!("Error: {e:?}"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FileSystemError {
    FileNotFound,
    FileNotOpen,
    NotOpenForRead,
    NotOpenForWrite,
    TooManyOpen,
    TooManyFiles,
    AlreadyOpen,
    DiskFull,
    FileTooBig,
    FilenameTooLong,
}

#[derive(Debug, Copy, Clone)]
pub struct FileInfo<const MAX_BLOCKS: usize, const BLOCK_SIZE: usize> {
    inode: Inode<MAX_BLOCKS, BLOCK_SIZE>,
    inode_num: usize,
    current_block: usize,
    offset: usize,
    writing: bool,
    block_buffer: [u8; BLOCK_SIZE],
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Inode<const MAX_BLOCKS: usize, const BLOCK_SIZE: usize> {
    bytes_stored: u16,
    blocks: [u8; MAX_BLOCKS],
}

const INODE_FULL_BLOCK: usize = 0;
const DATA_FULL_BLOCK: usize = INODE_FULL_BLOCK + 1;
const INODE_TABLE_START: usize = DATA_FULL_BLOCK + 1;

#[derive(core::fmt::Debug)]
pub struct FileSystem<
    const MAX_OPEN: usize,
    const BLOCK_SIZE: usize,
    const NUM_BLOCKS: usize,
    const MAX_FILE_BLOCKS: usize,
    const MAX_FILE_BYTES: usize,
    const MAX_FILES_STORED: usize,
    const MAX_FILENAME_BYTES: usize,
> {
    open: [Option<FileInfo<MAX_FILE_BLOCKS, BLOCK_SIZE>>; MAX_OPEN],
    disk: ramdisk::RamDisk<BLOCK_SIZE, NUM_BLOCKS>,
    block_buffer: [u8; BLOCK_SIZE],
    file_content_buffer: [u8; MAX_FILE_BYTES],
    open_inodes: [bool; MAX_FILES_STORED],
}

impl<
        const MAX_OPEN: usize,
        const BLOCK_SIZE: usize,
        const NUM_BLOCKS: usize,
        const MAX_FILE_BLOCKS: usize,
        const MAX_FILE_BYTES: usize,
        const MAX_FILES_STORED: usize,
        const MAX_FILENAME_BYTES: usize,
    >
    FileSystem<
        MAX_OPEN,
        BLOCK_SIZE,
        NUM_BLOCKS,
        MAX_FILE_BLOCKS,
        MAX_FILE_BYTES,
        MAX_FILES_STORED,
        MAX_FILENAME_BYTES,
    >
{
    pub fn new(disk: ramdisk::RamDisk<BLOCK_SIZE, NUM_BLOCKS>) -> Self {
        assert_eq!(MAX_FILE_BYTES, MAX_FILE_BLOCKS * BLOCK_SIZE);
        assert!(NUM_BLOCKS <= u8::MAX as usize);
        assert!(MAX_FILE_BYTES <= u16::MAX as usize);
        let block_bits = BLOCK_SIZE * 8;
        assert!(MAX_FILES_STORED <= block_bits);
        assert!(MAX_FILES_STORED <= u16::MAX as usize);
        let result = Self {
            open: [None; MAX_OPEN],
            disk,
            block_buffer: [0; BLOCK_SIZE],
            file_content_buffer: [0; MAX_FILE_BYTES],
            open_inodes: [false; MAX_FILES_STORED],
        };
        assert!(result.num_inode_blocks() * 2 < NUM_BLOCKS);
        assert!(result.num_data_blocks() <= block_bits);
        assert_eq!(
            result.num_data_blocks() + result.num_inode_blocks() + 2,
            NUM_BLOCKS
        );
        assert!(result.num_inode_entries() <= u16::MAX as usize);
        assert!(result.num_inode_blocks() <= MAX_FILE_BLOCKS);
        result
    }

    pub fn max_file_size(&self) -> usize {
        MAX_FILE_BLOCKS * BLOCK_SIZE
    }

    pub fn num_inode_bytes(&self) -> usize {
        2 + MAX_FILE_BLOCKS
    }

    pub fn inodes_per_block(&self) -> usize {
        BLOCK_SIZE / self.num_inode_bytes()
    }

    pub fn num_inode_blocks(&self) -> usize {
        MAX_FILES_STORED / self.inodes_per_block()
    }

    pub fn num_data_blocks(&self) -> usize {
        NUM_BLOCKS - self.num_inode_blocks() - 2
    }

    pub fn num_inode_entries(&self) -> usize {
        self.inodes_per_block() * self.num_inode_blocks() * self.num_inode_bytes()
    }

    pub fn first_data_block(&self) -> usize {
        2 + self.num_inode_blocks()
    }

    pub fn open_read(&mut self, filename: &str) -> FileSystemResult<usize> {
        todo!("Your code here");
    }

    pub fn open_create(&mut self, filename: &str) -> FileSystemResult<usize> {
        self.disk.read(0, &mut self.block_buffer);
        if self.block_buffer[0] % 2 != 1 { // means root directory hasn't been initailized
            self.block_buffer = [00000000; BLOCK_SIZE];
            self.block_buffer[0] = 00000001;
            let mut existblocks_buffer = [0 as u8; BLOCK_SIZE];
            self.disk.read(1, &mut existblocks_buffer);
            // existblocks_buffer = [00000000; BLOCK_SIZE];
            // existblocks_buffer[0] = 00111111;
            for i in 0..self.num_inode_blocks()+1 {
                existblocks_buffer[0] |= 1 << i;
            }
            self.disk.write(1, &existblocks_buffer);
            // I STILL NEED TO CREATE AN INODE??
        }
        else if false { // Check if an inode exists somehow
            todo!("This is for if there already exists an inode")
        }
        else {
            let mut first_open_index = 0;
            let mut spot = 0;
            for (i, byte) in self.block_buffer.iter().enumerate().take(BLOCK_SIZE) {
                if *byte != 255 as u8 {
                    first_open_index = i;
                    spot = byte.count_ones();
                }
            }
            self.block_buffer[first_open_index] |= 1 << spot;
            self.disk.write(0, &self.block_buffer);
            // Still need to do other stuff here
        }
        

        todo!("Your code here");
    }

    pub fn open_append(&mut self, filename: &str) -> FileSystemResult<usize> {
        todo!("Your code here");
    }

    pub fn close(&mut self, fd: usize) -> FileSystemResult<()> {
        todo!("Your code here");
    }

    pub fn read(&mut self, fd: usize, buffer: &mut [u8]) -> FileSystemResult<usize> {
        todo!("Your code here");
    }

    pub fn write(&mut self, fd: usize, buffer: &[u8]) -> FileSystemResult<()> {
        todo!("Your code here");
    }

}
