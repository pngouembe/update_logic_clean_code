use crate::{async_update::memory::MemoryMapping, reporting::UpdateError};

use super::software_archive::SoftwareArchive;

pub fn async_update(
    memory_mapping_path: &str,
    software_archive_path: &str,
) -> Result<(), UpdateError> {
    let memory_mapping = MemoryMapping::from(memory_mapping_path)?;

    let software_archive = SoftwareArchive::from(software_archive_path)?;

    software_archive.extract_logical_blocks(memory_mapping)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn async_update_test() {
        let result = async_update(
            "./resources/test/test_lb_cfg.json",
            "./resources/test/update_folder.zip",
        );

        assert_eq!(result, Ok(()))
    }
}
