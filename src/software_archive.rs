use std::{fmt, fs::File, io::Read};

use zip::{read::ZipFile, ZipArchive};

use crate::reporting::UpdateError;

const MANIFEST_XML_NAMESPACE: &str = "logical_blocks";

#[derive(Debug, Clone)]
pub struct LogicalBlock {
    id: String,
    name: String,
    signature: String,
    path_in_archive: String,
}
impl LogicalBlock {
    pub(crate) fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl fmt::Display for LogicalBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} logical block (id: 0x{}, signature: {})",
            self.name, self.id, self.signature
        )
    }
}

pub struct LogicalBlockReader<'a> {
    logical_block: LogicalBlock,
    file: ZipFile<'a>,
}

impl<'a> LogicalBlockReader<'a> {
    pub(crate) fn get_logical_block_id(&self) -> String {
        self.logical_block.get_id()
    }
}

impl<'a> Read for LogicalBlockReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl<'a> fmt::Display for LogicalBlockReader<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.logical_block.fmt(f)
    }
}

#[derive(Debug)]
pub struct SoftwareArchive {
    archive: ZipArchive<File>,
    logical_blocks: Vec<LogicalBlock>,
}

impl SoftwareArchive {
    pub fn from(archive_path: &str) -> Result<SoftwareArchive, UpdateError> {
        let zipfile = File::open(archive_path).unwrap();
        let archive = ZipArchive::new(zipfile).unwrap();
        let mut archive = SoftwareArchive {
            archive: archive,
            logical_blocks: vec![],
        };

        archive.index_logical_blocks()?;
        Ok(archive)
    }

    fn index_logical_blocks(&mut self) -> Result<(), UpdateError> {
        let index = self.get_index()?;
        let manifest = self.get_manifest(&index)?;
        self.index_logical_blocks_from_manifest_and_index(&manifest, &index)?;
        Ok(())
    }

    fn get_index(&mut self) -> Result<minidom::Element, UpdateError> {
        let index = self.get_file_content("index.xml")?;
        let index = index.parse().unwrap();
        Ok(index)
    }

    fn get_file_content(&mut self, relative_path: &str) -> Result<String, UpdateError> {
        let mut file = self.archive.by_name(relative_path).unwrap();
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).unwrap();
        Ok(file_content)
    }

    fn get_manifest(&mut self, index: &minidom::Element) -> Result<minidom::Element, UpdateError> {
        let manifest_path = self.get_manifest_path_from_index(&index)?;
        let manifest = self.get_file_content(&manifest_path)?;
        let manifest = manifest.parse().unwrap();
        Ok(manifest)
    }

    fn get_manifest_path_from_index(
        &self,
        index: &minidom::Element,
    ) -> Result<String, UpdateError> {
        let manifest_info = index
            .children()
            .find(|elem| elem.attr("short_name") == Some("update_manifest"))
            .unwrap();

        let manifest_path = manifest_info.get_child("path", "file_list").unwrap().text();

        Ok(manifest_path)
    }

    fn index_logical_blocks_from_manifest_and_index(
        &mut self,
        manifest: &minidom::Element,
        index: &minidom::Element,
    ) -> Result<(), UpdateError> {
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

            let path_in_archive = index
                .children()
                .find(|elem| elem.attr("short_name") == Some(&name))
                .unwrap()
                .get_child("path", "file_list")
                .unwrap()
                .text();

            self.logical_blocks.push(LogicalBlock {
                id: id,
                name: name,
                signature: signature,
                path_in_archive: path_in_archive,
            });
        }
        Ok(())
    }

    pub(crate) fn get_logical_block_reader(
        &mut self,
        logical_block: &LogicalBlock,
    ) -> LogicalBlockReader<'_> {
        LogicalBlockReader {
            logical_block: logical_block.clone(),
            file: self
                .archive
                .by_name(&logical_block.path_in_archive)
                .unwrap(),
        }
    }

    pub fn get_logical_blocks(&self) -> Vec<LogicalBlock> {
        self.logical_blocks.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_archive_test() {
        let archive = SoftwareArchive::from("./resources/test/update_folder.zip").unwrap();

        for logical_block in archive.get_logical_blocks() {
            println!("{}", logical_block)
        }
    }
}
