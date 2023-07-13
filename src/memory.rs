use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, Write},
};

use crate::software_archive;
use crate::{
    reporting::{LogicalBlockError, UpdateError},
    software_archive::LogicalBlockReader,
};

#[derive(Debug, Deserialize, PartialEq)]
pub struct LogicalBlockCfg {
    pub logical_blocks: Vec<LogicalBlock>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct LogicalBlock {
    pub name: String,
    pub id: String,
    pub destination: Banks,
}
impl LogicalBlock {
    fn get_location_from_bank(
        &self,
        targeted_bank: &str,
    ) -> Result<LogicalBlockLocation, UpdateError> {
        match targeted_bank {
            "bank_a" => Ok(self.destination.bank_a.clone()),
            "bank_b" => Ok(self.destination.bank_b.clone()),
            other => panic!("{} is not a supported bank", other),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Banks {
    pub bank_a: LogicalBlockLocation,
    pub bank_b: LogicalBlockLocation,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]

pub struct LogicalBlockLocation {
    path: String,
    offset: u64,
    size: usize,
}

impl LogicalBlockLocation {
    pub fn get_size(&self) -> usize {
        self.size
    }
}

pub struct LogicalBlockWriter<'a> {
    logical_block_destination: LogicalBlockLocation,
    logical_block_reader: LogicalBlockReader<'a>,
    destination_file: File,
}

impl<'a> LogicalBlockWriter<'a> {
    pub fn from(
        logical_block_reader: LogicalBlockReader<'a>,
        logical_block_destination: LogicalBlockLocation,
    ) -> Result<LogicalBlockWriter<'a>, UpdateError> {
        let mut file = File::options()
            .write(true)
            .open(&logical_block_destination.path)
            .unwrap();
        file.seek(std::io::SeekFrom::Start(logical_block_destination.offset))
            .unwrap();

        Ok(LogicalBlockWriter {
            logical_block_destination,
            logical_block_reader,
            destination_file: file,
        })
    }

    pub fn get_size(&self) -> usize {
        self.logical_block_destination.get_size()
    }

    pub fn get_destination(&self) -> LogicalBlockLocation {
        self.logical_block_destination.clone()
    }

    pub fn write(&mut self) -> Result<usize, UpdateError> {
        self.recursive_copy_to_file()
    }

    fn recursive_copy_to_file(&mut self) -> Result<usize, UpdateError> {
        let mut read_buffer = [0; 4096];
        let mut total_copied_bytes = 0;

        loop {
            let copied_bytes_count = self.copy_chunk(&mut read_buffer)?;
            if copied_bytes_count == 0 {
                break;
            } else {
                total_copied_bytes += copied_bytes_count;
            }
        }

        Ok(total_copied_bytes)
    }

    fn copy_chunk(&mut self, chunk_buffer: &mut [u8]) -> Result<usize, UpdateError> {
        let read_bytes = self.read_chunk_from_logical_block(chunk_buffer)?;

        let written_bytes = self.write_chunk_to_destination(&mut chunk_buffer[..read_bytes])?;

        match written_bytes == read_bytes {
            true => Ok(written_bytes),
            false => Err(UpdateError::LogicalBlockWrite(LogicalBlockError {
                logical_block_id: self.logical_block_reader.get_logical_block_id(),
                description: "todo!()".to_string(),
            })),
        }
    }

    fn read_chunk_from_logical_block(
        &mut self,
        chunk_buffer: &mut [u8],
    ) -> Result<usize, UpdateError> {
        match self.logical_block_reader.read(chunk_buffer) {
            Ok(n) => Ok(n),
            Err(_) => Err(UpdateError::LogicalBlockRead(LogicalBlockError {
                logical_block_id: self.logical_block_reader.get_logical_block_id(),
                description: "todo!()".to_string(),
            })),
        }
    }

    fn write_chunk_to_destination(
        &mut self,
        chunk_buffer: &mut [u8],
    ) -> Result<usize, UpdateError> {
        match self.destination_file.write(chunk_buffer) {
            Ok(n) => Ok(n),
            Err(_) => Err(UpdateError::LogicalBlockWrite(LogicalBlockError {
                logical_block_id: self.logical_block_reader.get_logical_block_id(),
                description: "todo!()".to_string(),
            })),
        }
    }
}
pub struct MemoryMapping {
    logical_blocks: HashMap<String, LogicalBlockLocation>,
}

impl MemoryMapping {
    pub fn from(mapping_path: &str) -> Result<MemoryMapping, UpdateError> {
        let mapping_file = File::open(mapping_path).unwrap();
        let lb_cfg: LogicalBlockCfg = serde_json::from_reader(mapping_file).unwrap();

        let targeted_bank = Self::get_target_bank()?;
        let target_bank_mapping =
            lb_cfg
                .logical_blocks
                .iter()
                .fold(HashMap::new(), |mut map, lb| {
                    let location = lb.get_location_from_bank(&targeted_bank).unwrap();

                    map.insert(lb.id.clone(), location);
                    map
                });

        Ok(MemoryMapping {
            logical_blocks: target_bank_mapping,
        })
    }

    pub fn get_logical_block_writer(
        &self,
        logical_block: &software_archive::LogicalBlock,
    ) -> Result<LogicalBlockLocation, UpdateError> {
        if let Some(location) = self.logical_blocks.get(&logical_block.get_id()) {
            Ok(location.clone())
        } else {
            Err(UpdateError::MissingLogicalBlock(LogicalBlockError {
                logical_block_id: logical_block.get_id(),
                description: "todo!()".to_string(),
            }))
        }
    }

    fn get_target_bank() -> Result<String, UpdateError> {
        Ok("bank_a".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_mapping_test() {
        let mapping = MemoryMapping::from("./resources/test/test_lb_cfg.json").unwrap();

        for (id, location) in mapping.logical_blocks {
            println!("id: {}, location: {:#?}", id, location);
        }
    }
}
