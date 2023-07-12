mod memory;
use memory::MemoryMapping;

mod software_archive;
use software_archive::SoftwareArchive;

mod reporting;
use reporting::UpdateError;

pub fn update_software(
    memory_mapping: MemoryMapping,
    new_software_archive: SoftwareArchive,
) -> Result<(), UpdateError> {
    for logical_block in new_software_archive.into_iter().as_mut_slice() {
        let logical_block_destination =
            memory_mapping.get_logical_block_location(&logical_block)?;

        logical_block_destination.write(logical_block)?;
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
