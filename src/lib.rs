#![cfg_attr(not(test), no_std)]

use embedded_io::{blocking::{Read, Write, Seek, ReadExactError}, SeekFrom, Io};
use core::cmp;

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

#[inline]
pub fn lba_to_u64(lba: u32) -> u64 {
    (lba as u64) * BLOCK_SIZE
}

/// Struct used for interfacing with partitions
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

impl <'a, IO> Partition<'a, IO> {
    #[inline]
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
            false => &mut buf[..available as usize]
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
            false => &buf[..available as usize]
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
            },
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

/// Used to store data about partitions
#[derive(Debug, Copy, Clone, Default)]
pub struct PartitionRecord {
    relative_sector: u32,
    total_sectors: u32,
}

impl PartitionRecord {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let relative_sector_array: [u8; 4] = bytes[RELATIVE_SECTOR_OFFSET..TOTAL_SECTORS_OFFSET].try_into().unwrap();
        let total_sectors_array: [u8; 4] = bytes[TOTAL_SECTORS_OFFSET..RECORD_LEN].try_into().unwrap();

        let relative_sector = u32::from_le_bytes(relative_sector_array);
        let total_sectors = u32::from_le_bytes(total_sectors_array);

        Self {
            relative_sector,
            total_sectors,
        }
    }

    pub fn get_start_pos(&self) -> u64 {
        lba_to_u64(self.relative_sector)
    }

    pub fn get_end_pos(&self) -> u64 {
        lba_to_u64(self.relative_sector) + lba_to_u64(self.total_sectors)
    }
}

/// Struct used to read the MBR
pub struct Mbr<IO: Read + Seek> {
    partitions: [PartitionRecord; RECORD_COUNT],
    io: IO,
}

impl<IO: Read + Seek> Mbr<IO> {
    /// Create a new MBR from an IO device
    pub fn new(mut io: IO) -> Result<Self, <IO as Io>::Error> {
        let mut partitions: [PartitionRecord; RECORD_COUNT] = [PartitionRecord::default(); RECORD_COUNT];
        let mut buffer: [u8; RECORD_LEN * RECORD_COUNT] = [0; RECORD_LEN * RECORD_COUNT];

        io.seek(SeekFrom::Start(RECORDS_START))?;
        io.read(&mut buffer)?;

        for i in 0..RECORD_COUNT {
            let buffer_i = i * RECORD_LEN;

            let record_slice = &buffer[buffer_i..buffer_i + RECORD_LEN];

            partitions[i] = PartitionRecord::from_bytes(&record_slice);
        }

        Ok(Self {
            partitions,
            io,
        })
    }

    /// Get a partition from the MBR
    pub fn get_partition(&mut self, num: usize) -> Result<Partition<IO>, IO::Error> {
        let record = self.partitions[num];

        Partition::new(record.get_start_pos(), record.get_end_pos(), &mut self.io)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use embedded_io::{adapters::FromStd, blocking::{Read, Seek}};

    use crate::{Mbr, BLOCK_SIZE};

    static TEST_STR_1: [u8;10] = *b"Partition1";
    static TEST_STR_2: [u8;10] = *b"Partition2";
    static TEST_STR_3: [u8;10] = *b"Partition3";
    static TEST_STR_4: [u8;10] = *b"Partition4";

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
        let img = FromStd::new(File::open("test.img").unwrap());

        let mut mbr = Mbr::new(img).unwrap();

        // Test partition 1
        let mut partition_1 = mbr.get_partition(0).unwrap();

        let mut buf: [u8;10] = [0;10];

        partition_1.read_exact(&mut buf[..9]).unwrap();
        partition_1.seek(embedded_io::SeekFrom::End(-1)).unwrap();
        partition_1.read_exact(&mut buf[9..]).unwrap();
        assert_eq!(partition_1.len(), 17 * BLOCK_SIZE);
        assert_eq!(buf, TEST_STR_1);
        drop(partition_1);

        // Test partition 2
        let mut partition_2 = mbr.get_partition(1).unwrap();

        partition_2.read_exact(&mut buf[..9]).unwrap();
        partition_2.seek(embedded_io::SeekFrom::End(-1)).unwrap();
        partition_2.read_exact(&mut buf[9..]).unwrap();
        assert_eq!(partition_2.len(), 33 * BLOCK_SIZE);
        assert_eq!(buf, TEST_STR_2);
        drop(partition_2);

        // Test partition 3
        let mut partition_3 = mbr.get_partition(2).unwrap();

        partition_3.read_exact(&mut buf[..9]).unwrap();
        partition_3.seek(embedded_io::SeekFrom::End(-1)).unwrap();
        partition_3.read_exact(&mut buf[9..]).unwrap();
        assert_eq!(partition_3.len(), 65 * BLOCK_SIZE);
        assert_eq!(buf, TEST_STR_3);
        drop(partition_3);

        // Test partition 2
        let mut partition_4 = mbr.get_partition(3).unwrap();

        partition_4.read_exact(&mut buf[..9]).unwrap();
        partition_4.seek(embedded_io::SeekFrom::End(-1)).unwrap();
        partition_4.read_exact(&mut buf[9..]).unwrap();
        assert_eq!(partition_4.len(), 84 * BLOCK_SIZE);
        assert_eq!(buf, TEST_STR_4);
    }
}
