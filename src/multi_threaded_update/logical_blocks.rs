use std::{
    fmt,
    fs::File,
    io::{Read, Seek, Write},
};

use crate::{
    multi_threaded_update::memory::LogicalBlockDestination,
    reporting::{LogicalBlockError, UpdateError},
};

pub struct LogicalBlock<'a> {
    pub id: String,
    pub name: String,
    pub signature: String,
    pub source: LogicalBlockSource<'a>,
    pub destination: LogicalBlockDestination,
}

pub struct LogicalBlockSource<'a> {
    pub file: Box<dyn Read + Send + 'a>,
}

impl<'a> LogicalBlock<'a> {
    pub fn write(&mut self) -> Result<usize, UpdateError> {
        let mut read_buffer = [0; 4096];
        let mut total_copied_bytes = 0;

        let mut file = File::options()
            .write(true)
            .open(&self.destination.get_path())
            .unwrap();
        file.seek(std::io::SeekFrom::Start(self.destination.get_offset()))
            .unwrap();

        loop {
            let copied_bytes_count = self.copy_chunk(&mut read_buffer, &mut file)?;
            if copied_bytes_count == 0 {
                break;
            } else {
                total_copied_bytes += copied_bytes_count;
            }
        }

        match total_copied_bytes == self.destination.get_size() {
            true => Ok(total_copied_bytes),
            false => Err(UpdateError::LogicalBlockWrite(LogicalBlockError {
                logical_block_id: self.id.clone(),
                description: "todo!()".to_string(),
            })),
        }
    }

    fn copy_chunk(
        &mut self,
        chunk_buffer: &mut [u8],
        file: &mut File,
    ) -> Result<usize, UpdateError> {
        let read_bytes = self.read_chunk_from_logical_block(chunk_buffer)?;

        let written_bytes = self.write_chunk_in_file(&mut chunk_buffer[..read_bytes], file)?;

        match written_bytes == read_bytes {
            true => Ok(written_bytes),
            false => Err(UpdateError::LogicalBlockWrite(LogicalBlockError {
                logical_block_id: self.id.clone(),
                description: "todo!()".to_string(),
            })),
        }
    }

    fn read_chunk_from_logical_block(
        &mut self,
        chunk_buffer: &mut [u8],
    ) -> Result<usize, UpdateError> {
        match self.source.file.read(chunk_buffer) {
            Ok(n) => Ok(n),
            Err(_) => Err(UpdateError::LogicalBlockRead(LogicalBlockError {
                logical_block_id: self.id.clone(),
                description: "todo!()".to_string(),
            })),
        }
    }

    fn write_chunk_in_file(
        &mut self,
        chunk_buffer: &mut [u8],
        file: &mut File,
    ) -> Result<usize, UpdateError> {
        match file.write(chunk_buffer) {
            Ok(n) => Ok(n),
            Err(_) => Err(UpdateError::LogicalBlockWrite(LogicalBlockError {
                logical_block_id: self.id.clone(),
                description: "todo!()".to_string(),
            })),
        }
    }
}

impl<'a> fmt::Display for LogicalBlock<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} logical block (id: 0x{}, signature: {})",
            self.name, self.id, self.signature
        )
    }
}
