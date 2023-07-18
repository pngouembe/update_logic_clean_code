use std::fs::File;

use memmap2::Mmap;
use piz::{
    read::{as_tree, FileTree},
    ZipArchive,
};
use tokio::runtime::Runtime;

use crate::{async_update::memory::MemoryMapping, reporting::UpdateError};

use super::logical_blocks::{LogicalBlock, LogicalBlockSource};

const MANIFEST_XML_NAMESPACE: &str = "logical_blocks";

pub struct SoftwareArchive {
    archive_bytes: Mmap,
}

impl SoftwareArchive {
    pub fn from(archive_path: &str) -> Result<SoftwareArchive, UpdateError> {
        let archive_bytes = Self::read_archive(archive_path)?;
        Ok(SoftwareArchive { archive_bytes })
    }

    fn read_archive(archive_path: &str) -> Result<Mmap, UpdateError> {
        let zip_file = File::open(archive_path).unwrap();
        Ok(unsafe { Mmap::map(&zip_file).unwrap() })
    }

    pub fn extract_logical_blocks(&self, memory_mapping: MemoryMapping) -> Result<(), UpdateError> {
        let archive = self.get_archive()?;

        let logical_blocks = self.get_logical_blocks(&archive, memory_mapping)?;

        self.write_logical_blocks(logical_blocks)
    }

    fn get_archive(&self) -> Result<ZipArchive<'_>, UpdateError> {
        Ok(ZipArchive::new(&self.archive_bytes).unwrap())
    }

    fn get_logical_blocks<'a>(
        &'a self,
        archive: &'a ZipArchive<'_>,
        memory_mapping: MemoryMapping,
    ) -> Result<Vec<LogicalBlock>, UpdateError> {
        let index = self.read_archive_index(archive)?;
        let manifest = self.create_update_manifest(archive, index)?;

        self.get_logical_blocks_from_manifest_and_memory_map(manifest, memory_mapping, archive)
    }

    fn read_archive_index(
        &self,
        archive: &ZipArchive<'_>,
    ) -> Result<minidom::Element, UpdateError> {
        let index = self.read_file_content(archive, "index.xml").unwrap();
        Ok(index.parse().unwrap())
    }

    fn read_file_content(
        &self,
        archive: &ZipArchive<'_>,
        path_in_archive: &str,
    ) -> Result<String, UpdateError> {
        let tree = as_tree(archive.entries()).unwrap();
        let metadata = tree.lookup(path_in_archive).unwrap();

        let mut reader = archive.read(metadata).unwrap();

        let mut file_content = String::new();

        reader.read_to_string(&mut file_content).unwrap();

        Ok(file_content)
    }

    fn create_update_manifest(
        &self,
        archive: &ZipArchive<'_>,
        index: minidom::Element,
    ) -> Result<minidom::Element, UpdateError> {
        let manifest_path = self.get_manifest_path(&index)?;
        let manifest = self.read_file_content(archive, &manifest_path)?;

        let manifest = manifest.parse().unwrap();

        self.get_manifest_with_logical_block_paths(manifest, index)
    }

    fn get_manifest_path(&self, index: &minidom::Element) -> Result<String, UpdateError> {
        let manifest_info = index
            .children()
            .find(|elem| elem.attr("short_name") == Some("update_manifest"))
            .unwrap();

        let manifest_path = manifest_info.get_child("path", "file_list").unwrap().text();

        Ok(manifest_path)
    }

    fn get_manifest_with_logical_block_paths(
        &self,
        mut manifest: minidom::Element,
        index: minidom::Element,
    ) -> Result<minidom::Element, UpdateError> {
        for elem in manifest.children_mut() {
            let name = elem
                .get_child("short_name", MANIFEST_XML_NAMESPACE)
                .unwrap()
                .text();

            let path_in_archive = index
                .children()
                .find(|elem| elem.attr("short_name") == Some(&name))
                .unwrap()
                .get_child("path", "file_list")
                .unwrap()
                .text();

            let path_node =
                elem.append_child(minidom::Element::bare("path", MANIFEST_XML_NAMESPACE));
            path_node.append_node(minidom::Node::Text(path_in_archive))
        }

        Ok(manifest)
    }

    fn get_logical_blocks_from_manifest_and_memory_map<'a>(
        &'a self,
        manifest: minidom::Element,
        memory_mapping: MemoryMapping,
        archive: &'a ZipArchive<'_>,
    ) -> Result<Vec<LogicalBlock<'a>>, UpdateError> {
        let mut logical_blocks = Vec::new();

        let tree = as_tree(archive.entries()).unwrap();

        for elem in manifest.children() {
            let id = elem.get_child("id", MANIFEST_XML_NAMESPACE).unwrap().text();

            let name = elem
                .get_child("short_name", MANIFEST_XML_NAMESPACE)
                .unwrap()
                .text();

            let signature = elem
                .get_child("signature", MANIFEST_XML_NAMESPACE)
                .unwrap()
                .text();

            let path_in_archive = elem
                .get_child("path", MANIFEST_XML_NAMESPACE)
                .unwrap()
                .text();

            let metadata = tree.lookup(path_in_archive).unwrap();
            let logical_block_reader = archive.read(metadata).unwrap();

            let logical_block_source = LogicalBlockSource {
                file: logical_block_reader,
            };

            let logical_block_destination = memory_mapping
                .get_logical_block_destination(&id)
                .unwrap()
                .clone();

            logical_blocks.push(LogicalBlock {
                id,
                name,
                signature,
                source: logical_block_source,
                destination: logical_block_destination,
            })
        }
        Ok(logical_blocks)
    }

    fn write_logical_blocks(
        &self,
        logical_blocks: Vec<LogicalBlock<'_>>,
    ) -> Result<(), UpdateError> {
        let rt = Runtime::new().unwrap();

        rt.block_on(self.async_write_logical_blocks(logical_blocks))
    }

    async fn async_write_logical_blocks(
        &self,
        mut logical_blocks: Vec<LogicalBlock<'_>>,
    ) -> Result<(), UpdateError> {
        for logical_block in logical_blocks.iter_mut() {
            logical_block.write().await?;
            logical_block.verify().await?;
        }
        Ok(())
    }
}
