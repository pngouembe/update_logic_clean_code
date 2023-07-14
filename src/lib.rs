mod memory;
use memory::{LogicalBlockLocation, MemoryMapping};

mod software_archive;
use software_archive::{LogicalBlockReader, SoftwareArchive};

mod reporting;
use reporting::UpdateError;

mod crypto;
use crypto::LogicalBlockVerifier;

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

        write_logical_block(logical_block_reader, &logical_block_destination)?;

        verify_logical_block(logical_block_destination, logical_block_info)?;
    }
    Ok(())
}

fn write_logical_block(
    logical_block_reader: LogicalBlockReader<'_>,
    logical_block_destination: &LogicalBlockLocation,
) -> Result<(), UpdateError> {
    let logical_block_info = logical_block_reader.get_logical_block_info().clone();
    let mut logical_block_writer =
        LogicalBlockWriter::from(logical_block_reader, logical_block_destination.clone())?;

    println!(
        "Copy: {:#?}\nIn: {:#?}",
        logical_block_info,
        logical_block_writer.get_destination()
    );

    let bytes_count = logical_block_writer.write()?;

    match bytes_count == logical_block_writer.get_size() {
        true => Ok(()),
        false => Err(UpdateError::LogicalBlockSize(LogicalBlockError {
            logical_block_id: logical_block_info.get_id(),
            description: "todo!()".to_string(),
        })),
    }
}

fn verify_logical_block(
    logical_block_destination: LogicalBlockLocation,
    logical_block_info: software_archive::LogicalBlockInfo,
) -> Result<(), UpdateError> {
    let logical_block_verifier =
        LogicalBlockVerifier::from(logical_block_destination, logical_block_info.clone());

    if logical_block_verifier.verify()? {
        return Ok(());
    } else {
        return Err(UpdateError::VerificationError(LogicalBlockError {
            logical_block_id: logical_block_info.get_id(),
            description: "todo!()".to_string(),
        }));
    }
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
