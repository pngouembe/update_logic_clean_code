use serde::Deserialize;
use std::{collections::HashMap, fs::File};

use crate::reporting::UpdateError;

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
    ) -> Result<LogicalBlockDestination, UpdateError> {
        match targeted_bank {
            "bank_a" => Ok(self.destination.bank_a.clone()),
            "bank_b" => Ok(self.destination.bank_b.clone()),
            other => panic!("{} is not a supported bank", other),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Banks {
    pub bank_a: LogicalBlockDestination,
    pub bank_b: LogicalBlockDestination,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]

pub struct LogicalBlockDestination {
    path: String,
    offset: u64,
    size: usize,
}

impl LogicalBlockDestination {
    pub fn get_path(&self) -> &str {
        self.path.as_ref()
    }

    pub fn get_offset(&self) -> u64 {
        self.offset
    }

    pub fn get_size(&self) -> usize {
        self.size
    }
}

pub struct MemoryMapping {
    logical_blocks: HashMap<String, LogicalBlockDestination>,
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

    pub fn get_logical_block_destination(
        &self,
        logical_block_id: &str,
    ) -> Result<&LogicalBlockDestination, UpdateError> {
        Ok(self.logical_blocks.get(logical_block_id).unwrap())
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
