//! # *simple crate to interface between a disk and it's partitions*
//!
//! This crate is especially designed to provide an interface between a disk
//! and a file system library, where both are able to implement embedded_io.
//!
//! Dead simple, as it should be.
//!
//! # Usage
//!
//! This crate can be used by adding `ape-mbr` to the dependencies in your
//! project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! ape-mbr = "0.1.0"
//! ```
//!
//! # Examples
//!
//! Here's `ape-mbr` being coupled with `ape-fatfs`
//!
//! ```rust
//! use std::io::prelude::*;
//! use ape_fatfs::{
//!     fs::{
//!         FsOptions,
//!         FileSystem,
//!     },
//!     io::{
//!         StdIoWrapper
//!     }
//! };
//!
//! use ape_mbr::{
//!     PartitionId,
//!     MBR,
//! };
//!
//! fn main() {
//!     # std::fs::copy("resources/test2.img", "test.img").unwrap();
//!     // Initialize the MBR
//!     let img_file = std::fs::OpenOptions::new().read(true).write(true)
//!         .open("test.img").unwrap();
//!
//!     let img_file = StdIoWrapper::new(img_file);
//!    
//!     let mut mbr = MBR::new(img_file).unwrap();
//!     let mut p1 = mbr.get_partition(PartitionId::One).unwrap();
//!     
//!     let fs = FileSystem::new(p1, FsOptions::new()).unwrap();
//!     let root_dir = fs.root_dir();
//!
//!     // Write a file
//!     root_dir.create_dir("foo").unwrap();
//!     let mut file = root_dir.create_file("foo/hello.txt").unwrap();
//!     file.truncate().unwrap();
//!     file.write_all(b"Hello World!").unwrap();
//!
//!     // Read a directory
//!     let dir = root_dir.open_dir("foo").unwrap();
//!     for r in dir.iter() {
//!         let entry = r.unwrap();
//!         println!("{}", entry.file_name());
//!     }
//!     # std::fs::remove_file("test.img").unwrap();
//! }
//! ```
#![cfg_attr(not(test), no_std)]

use core::cmp;
use embedded_io::{
    blocking::{Read, Seek, Write},
    Io, SeekFrom,
};
use types::PartitionType;

pub mod types;

/// Length of each record in bytes
pub const RECORD_LEN: usize = 16;
/// Number of record in MBR
pub const RECORD_COUNT: usize = 4;
/// Size of blocks in bytes
pub const BLOCK_SIZE: u64 = 512;
/// Offset to the start of the partition records
pub const RECORDS_START: u64 = 0x1be;
/// Offset of the relative sector field in a partition record
pub const RELATIVE_SECTOR_OFFSET: usize = 8;
/// Offset of the total sectors field in a partition record
pub const TOTAL_SECTORS_OFFSET: usize = 12;
/// Offset of the system id field in a partition record
pub const SYSTEM_ID_OFFSET: usize = 4;
/// Offset of the boot indicator flag in a partition record
pub const BOOT_FLAG_OFFSET: usize = 0;

/// ID of each partition
#[repr(usize)]
pub enum PartitionId {
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
}

#[inline]
/// Convert an LBA address to a u64
pub fn lba_to_u64(lba: u32) -> u64 {
    (lba as u64) * BLOCK_SIZE
}

/// Used to interface with partitions
pub struct Partition<'a, IO> {
    start_pos: u64,
    end_pos: u64,
    pos: u64,
    io: &'a mut IO,
}

impl<'a, IO: Io + Seek> Partition<'a, IO> {
    /// Create a new partition given the start and end position
    pub fn new(start_pos: u64, end_pos: u64, io: &'a mut IO) -> Result<Self, <Self as Io>::Error> {
        // Seek to the start of the partition
        io.seek(SeekFrom::Start(start_pos))?;

        Ok(Self {
            start_pos,
            end_pos,
            pos: 0,
            io,
        })
    }
}

impl<'a, IO> Partition<'a, IO> {
    #[inline]
    /// Get the length of the partition in bytes
    pub fn len(&self) -> u64 {
        self.end_pos - self.start_pos
    }
}

impl<'a, IO: Io> Io for Partition<'a, IO> {
    type Error = IO::Error;
}

impl<'a, IO: Read> Read for Partition<'a, IO> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // Limit the amount of data available to read to the size of the partition
        let available = self.len() - self.pos;

        let buf_slice = match (buf.len() as u64) < available {
            true => buf,
            false => &mut buf[..available as usize],
        };

        self.pos += buf_slice.len() as u64;

        self.io.read(buf_slice)
    }
}

impl<'a, IO: Write> Write for Partition<'a, IO> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        // Limit the amount of data available to write to the size of the partition
        let available = self.len() - self.pos;

        let buf_slice = match (buf.len() as u64) < available {
            true => buf,
            false => &buf[..available as usize],
        };

        self.pos += buf_slice.len() as u64;

        self.io.write(buf_slice)
    }

    #[inline]
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.io.flush()
    }
}

impl<'a, IO: Seek> Seek for Partition<'a, IO> {
    fn seek(&mut self, pos: embedded_io::SeekFrom) -> Result<u64, Self::Error> {
        self.pos = match pos {
            SeekFrom::Start(pos) => {
                // Ensure that we don't go past the partition boundries
                cmp::min(pos, self.len())
            }
            SeekFrom::Current(pos) => {
                // Ensure that we don't go past the partition boundries
                cmp::max(cmp::min((pos as i64) + pos, self.len() as i64), 0) as u64
            }
            SeekFrom::End(pos) => {
                // Ensure that we don't go past the partition boundries
                cmp::max(cmp::min((self.len() as i64) + pos, self.len() as i64), 0) as u64
            }
        };

        self.io.seek(SeekFrom::Start(self.start_pos + self.pos))?;

        Ok(self.pos)
    }
}

/// Used to store data about partitions in the MBR
#[derive(Debug, Copy, Clone, Default)]
pub struct PartitionRecord {
    relative_sector: u32,
    total_sectors: u32,
    partition_type: PartitionType,
    boot_flag: bool,
}

impl PartitionRecord {
    /// Create a partition record from bytes
    pub fn from_bytes(bytes: &[u8; RECORD_LEN]) -> Self {
        let relative_sector_array: [u8; 4] = bytes[RELATIVE_SECTOR_OFFSET..TOTAL_SECTORS_OFFSET]
            .try_into()
            .unwrap();
        let total_sectors_array: [u8; 4] =
            bytes[TOTAL_SECTORS_OFFSET..RECORD_LEN].try_into().unwrap();

        let relative_sector = u32::from_le_bytes(relative_sector_array);
        let total_sectors = u32::from_le_bytes(total_sectors_array);
        
        let system_id: u8 = bytes[SYSTEM_ID_OFFSET];
        let boot_flag: bool = bytes[BOOT_FLAG_OFFSET] == 0x80;

        Self {
            relative_sector,
            total_sectors,
            partition_type: system_id.try_into().unwrap(),
            boot_flag,
        }
    }

    #[inline]
    /// Get the starting position of a partition
    pub fn get_start_pos(&self) -> u64 {
        lba_to_u64(self.relative_sector)
    }

    #[inline]
    /// Get the end position of a partition
    pub fn get_end_pos(&self) -> u64 {
        lba_to_u64(self.relative_sector) + lba_to_u64(self.total_sectors)
    }

    #[inline]
    /// Get the type of a partition
    pub fn get_partition_type(&self) -> PartitionType {
        self.partition_type
    }

    #[inline]
    /// Check to see if the partition's boot flag is set
    pub fn is_bootable(&self) -> bool {
        self.boot_flag
    }
}

/// Used to grab partitions from the MBR
pub struct MBR<IO: Read + Seek> {
    partitions: [PartitionRecord; RECORD_COUNT],
    io: IO,
}

impl<IO: Read + Seek> MBR<IO> {
    /// Create a new MBR from anything that implements embedded_io
    pub fn new(mut io: IO) -> Result<Self, <IO as Io>::Error> {
        let mut partitions: [PartitionRecord; RECORD_COUNT] =
            [PartitionRecord::default(); RECORD_COUNT];
        let mut buffer: [u8; RECORD_LEN * RECORD_COUNT] = [0; RECORD_LEN * RECORD_COUNT];

        io.seek(SeekFrom::Start(RECORDS_START))?;
        io.read(&mut buffer)?;

        for i in 0..RECORD_COUNT {
            let buffer_i = i * RECORD_LEN;

            let record_slice = &buffer[buffer_i..buffer_i + RECORD_LEN];

            partitions[i] = PartitionRecord::from_bytes(record_slice.try_into().unwrap());
        }

        Ok(Self { partitions, io })
    }

    #[inline]
    /// Get a partition from the MBR
    pub fn get_partition(&mut self, id: PartitionId) -> Result<Partition<IO>, IO::Error> {
        let record = self.partitions[id as usize];

        Partition::new(record.get_start_pos(), record.get_end_pos(), &mut self.io)
    }

    #[inline]
    /// Get the partition type from the MBR
    pub fn get_partition_type(&self, id: PartitionId) -> PartitionType {
        let record = self.partitions[id as usize];

        record.get_partition_type()
    }

    #[inline]
    /// Check if a partition is bootable in the MBR
    pub fn is_partition_bootable(&self, id: PartitionId) -> bool {
        let record = self.partitions[id as usize];

        record.is_bootable()
    }
}

#[cfg(test)]
mod tests {
    use core::panic::AssertUnwindSafe;
    use std::{io::Cursor, panic};

    use ape_fatfs::{fs::{FileSystem, FsOptions, FatType}, io::StdIoWrapper};
    use embedded_io::{
        adapters::FromStd,
        blocking::{Read, Seek, Write},
    };

    use crate::*;

    static TEST_IMG_1: &[u8] = include_bytes!("../resources/test1.img");
    static TEST_IMG_2: &[u8] = include_bytes!("../resources/test2.img");
    static TEST_STR_1: [u8; 10] = *b"Partition1";
    static TEST_STR_2: [u8; 10] = *b"Partition2";
    static TEST_STR_3: [u8; 10] = *b"Partition3";
    static TEST_STR_4: [u8; 10] = *b"Partition4";

    #[test]
    /// The dummy image is a four partition image with "Partition" witten to the
    /// start of each partition and the partition number written to the end
    /// each partition
    ///
    /// Partitions sizes in sectors:
    ///     First partition size: 17
    ///     Second partition size: 33
    ///     Third partition size: 65
    ///     Fourth partition size: 84
    fn test_dummy_img() {
        let img = FromStd::new(Cursor::new(TEST_IMG_1.to_vec()));

        let mut mbr = MBR::new(img).unwrap();

        // Test partition 1
        let mut partition_1 = mbr.get_partition(PartitionId::One).unwrap();

        let mut buf: [u8; 10] = [0; 10];

        partition_1.read_exact(&mut buf[..9]).unwrap();
        partition_1.seek(embedded_io::SeekFrom::End(-1)).unwrap();
        partition_1.read_exact(&mut buf[9..]).unwrap();
        assert_eq!(partition_1.len(), 17 * BLOCK_SIZE);
        assert_eq!(buf, TEST_STR_1);
        drop(partition_1);

        // Test partition 2
        let mut partition_2 = mbr.get_partition(PartitionId::Two).unwrap();

        partition_2.read_exact(&mut buf[..9]).unwrap();
        partition_2.seek(embedded_io::SeekFrom::End(-1)).unwrap();
        partition_2.read_exact(&mut buf[9..]).unwrap();
        assert_eq!(partition_2.len(), 33 * BLOCK_SIZE);
        assert_eq!(buf, TEST_STR_2);
        drop(partition_2);

        // Test partition 3
        let mut partition_3 = mbr.get_partition(PartitionId::Three).unwrap();

        partition_3.read_exact(&mut buf[..9]).unwrap();
        partition_3.seek(embedded_io::SeekFrom::End(-1)).unwrap();
        partition_3.read_exact(&mut buf[9..]).unwrap();
        assert_eq!(partition_3.len(), 65 * BLOCK_SIZE);
        assert_eq!(buf, TEST_STR_3);
        drop(partition_3);

        // Test partition 2
        let mut partition_4 = mbr.get_partition(PartitionId::Four).unwrap();

        partition_4.read_exact(&mut buf[..9]).unwrap();
        partition_4.seek(embedded_io::SeekFrom::End(-1)).unwrap();
        partition_4.read_exact(&mut buf[9..]).unwrap();
        assert_eq!(partition_4.len(), 84 * BLOCK_SIZE);
        assert_eq!(buf, TEST_STR_4);
    }

    #[test]
    /// The "real" image is a three partition image designed to simulate a real
    /// drive
    ///
    /// Partition layout is as follows:
    ///     fat12, 2000 sectors, boot flag set
    ///     fat16, 5000 sectors
    ///     fat32, 68000 sectors
    fn test_real_img() {
        let img = StdIoWrapper::new(Cursor::new(TEST_IMG_2.to_vec()));

        let mut mbr = MBR::new(img).unwrap();

        // Test partition 1
        assert_eq!(mbr.get_partition_type(PartitionId::One), PartitionType::Fat12);
        assert!(mbr.is_partition_bootable(PartitionId::One));

        {
            let partition = mbr.get_partition(PartitionId::One).unwrap();

            let fs = FileSystem::new(partition, FsOptions::new()).unwrap();

            assert_eq!(fs.fat_type(), FatType::Fat12);
        }

        // Test partition 2
        assert_eq!(mbr.get_partition_type(PartitionId::Two), PartitionType::Fat16);
        assert!(!mbr.is_partition_bootable(PartitionId::Two));

        {
            let partition = mbr.get_partition(PartitionId::Two).unwrap();

            let fs = FileSystem::new(partition, FsOptions::new()).unwrap();

            assert_eq!(fs.fat_type(), FatType::Fat16);
        }

        // Test partition 3
        assert_eq!(mbr.get_partition_type(PartitionId::Three), PartitionType::W95Fat32);
        assert!(!mbr.is_partition_bootable(PartitionId::Three));

        {
            let partition = mbr.get_partition(PartitionId::Three).unwrap();

            let fs = FileSystem::new(partition, FsOptions::new()).unwrap();

            assert_eq!(fs.fat_type(), FatType::Fat32);
        }
    }

    #[test]
    /// Ensure that we cannot read or write past the end of the partition
    fn test_bounds() {
        let img = FromStd::new(Cursor::new(TEST_IMG_1.to_vec()));

        let mut mbr = MBR::new(img).unwrap();

        let mut buf: [u8; 10] = [0; 10];
        let mut partition_1 = mbr.get_partition(PartitionId::One).unwrap();

        partition_1.seek(embedded_io::SeekFrom::End(0)).unwrap();

        partition_1.read_exact(&mut buf).unwrap_err();
        // write_all panics on failure
        assert!(panic::catch_unwind(AssertUnwindSafe(|| {
            partition_1.write_all(&buf).unwrap_err();
        }))
        .is_err());
    }
}
