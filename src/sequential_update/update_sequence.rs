use crate::reporting::UpdateError;
use crate::sequential_update::crypto::LogicalBlockVerifier;
use crate::sequential_update::memory::{LogicalBlockDestination, MemoryMapping};
use crate::sequential_update::software_archive::{
    LogicalBlockInfo, LogicalBlockReader, SoftwareArchive,
};
use crate::{reporting::LogicalBlockError, sequential_update::memory::LogicalBlockWriter};

pub fn sequencial_update(
    memory_mapping_path: &str,
    software_archive_path: &str,
) -> Result<(), UpdateError> {
    let mut new_software_archive = SoftwareArchive::from(software_archive_path).unwrap();

    let memory_mapping = MemoryMapping::from(memory_mapping_path).unwrap();

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
    logical_block_destination: &LogicalBlockDestination,
) -> Result<(), UpdateError> {
    let logical_block_info = logical_block_reader.get_logical_block_info().clone();
    let mut logical_block_writer =
        LogicalBlockWriter::from(logical_block_reader, logical_block_destination.clone())?;

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
    logical_block_destination: LogicalBlockDestination,
    logical_block_info: LogicalBlockInfo,
) -> Result<(), UpdateError> {
    let logical_block_verifier =
        LogicalBlockVerifier::from(logical_block_destination, logical_block_info.clone());

    if logical_block_verifier.verify()? {
        Ok(())
    } else {
        Err(UpdateError::VerificationError(LogicalBlockError {
            logical_block_id: logical_block_info.get_id(),
            description: "todo!()".to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequencial_update_test() {
        let result = sequencial_update(
            "./resources/test/test_lb_cfg.json",
            "./resources/test/update_folder.zip",
        );

        assert_eq!(result, Ok(()))
    }
}
