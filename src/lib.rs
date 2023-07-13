mod memory;
use memory::MemoryMapping;

mod software_archive;
use software_archive::SoftwareArchive;

mod reporting;
use reporting::UpdateError;

use crate::{memory::LogicalBlockWriter, reporting::LogicalBlockError};

pub fn update_software(
    memory_mapping: MemoryMapping,
    mut new_software_archive: SoftwareArchive,
) -> Result<(), UpdateError> {
    for logical_block_info in new_software_archive.get_logical_blocks_info() {
        let logical_block_reader =
            new_software_archive.get_logical_block_reader(&logical_block_info);
        let logical_block_destination =
            memory_mapping.get_logical_block_writer(&logical_block_info)?;

        let mut logical_block_writer =
            LogicalBlockWriter::from(logical_block_reader, logical_block_destination)?;

        println!(
            "Copy: {:#?}\nIn: {:#?}",
            logical_block_info,
            logical_block_writer.get_destination()
        );

        let bytes_count = logical_block_writer.write()?;

        if bytes_count != logical_block_writer.get_size() {
            return Err(UpdateError::LogicalBlockSize(LogicalBlockError {
                logical_block_id: logical_block_info.get_id(),
                description: "todo!()".to_string(),
            }));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_update_test() {
        let archive = SoftwareArchive::from("./resources/test/update_folder.zip").unwrap();

        let mapping = MemoryMapping::from("./resources/test/test_lb_cfg.json").unwrap();

        update_software(mapping, archive).unwrap();
    }
}
