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
    pub fn write(&self, logical_block_reader: &mut LogicalBlockReader) -> Result<(), UpdateError> {
        let mut file = File::options().write(true).open(&self.path).unwrap();
        file.seek(std::io::SeekFrom::Start(self.offset)).unwrap();

        self.recursive_copy_to_file(&mut file, logical_block_reader)
    }

    fn recursive_copy_to_file(
        &self,
        file: &mut File,
        logical_block_reader: &mut LogicalBlockReader,
    ) -> Result<(), UpdateError> {
        let mut read_buffer = [0; 4096];

        loop {
            match logical_block_reader.read(&mut read_buffer) {
                Ok(0) => break, // finished reading
                Ok(bytes_count) => {
                    let written_bytes_count = file.write(&read_buffer[..bytes_count]).unwrap();
                    if written_bytes_count != bytes_count {
                        return Err(UpdateError::LogicalBlockWriteError(LogicalBlockError {
                            logical_block_id: logical_block_reader.get_logical_block_id(),
                            description: "todo!()".to_string(),
                        }));
                    }
                }
                Err(_) => {
                    return Err(UpdateError::LogicalBlockWriteError(LogicalBlockError {
                        logical_block_id: logical_block_reader.get_logical_block_id(),
                        description: "todo!()".to_string(),
                    }))
                }
            }
        }
        Ok(())
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

    pub fn get_logical_block_location(
        &self,
        logical_block: &software_archive::LogicalBlock,
    ) -> Result<LogicalBlockLocation, UpdateError> {
        if let Some(location) = self.logical_blocks.get(&logical_block.get_id()) {
            Ok(location.clone())
        } else {
            Err(UpdateError::MissingLogicalBlockError(LogicalBlockError {
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
