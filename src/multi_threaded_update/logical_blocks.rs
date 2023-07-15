use std::{
    fmt,
    fs::File,
    io::{Read, Seek, Write},
};

use base64::{engine::general_purpose, Engine};
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Public},
    rsa::Padding,
    sign::{RsaPssSaltlen, Verifier},
};

use crate::{
    multi_threaded_update::memory::LogicalBlockDestination,
    reporting::{LogicalBlockError, UpdateError},
};

pub struct LogicalBlock<'a> {
    pub id: String,
    pub name: String,
    pub signature: String,
    pub source: LogicalBlockSource<'a>,
    pub destination: LogicalBlockDestination,
}

pub struct LogicalBlockSource<'a> {
    pub file: Box<dyn Read + Send + 'a>,
}

impl<'a> LogicalBlock<'a> {
    pub fn write(&mut self) -> Result<usize, UpdateError> {
        let mut read_buffer = [0; 4096];
        let mut total_copied_bytes = 0;

        let mut file = File::options()
            .write(true)
            .open(&self.destination.get_path())
            .unwrap();
        file.seek(std::io::SeekFrom::Start(self.destination.get_offset()))
            .unwrap();

        loop {
            let copied_bytes_count = self.copy_chunk(&mut read_buffer, &mut file)?;
            if copied_bytes_count == 0 {
                break;
            } else {
                total_copied_bytes += copied_bytes_count;
            }
        }

        match total_copied_bytes == self.destination.get_size() {
            true => Ok(total_copied_bytes),
            false => Err(UpdateError::LogicalBlockWrite(LogicalBlockError {
                logical_block_id: self.id.clone(),
                description: "todo!()".to_string(),
            })),
        }
    }

    fn copy_chunk(
        &mut self,
        chunk_buffer: &mut [u8],
        file: &mut File,
    ) -> Result<usize, UpdateError> {
        let read_bytes = self.read_chunk_from_logical_block(chunk_buffer)?;

        let written_bytes = self.write_chunk_in_file(&mut chunk_buffer[..read_bytes], file)?;

        match written_bytes == read_bytes {
            true => Ok(written_bytes),
            false => Err(UpdateError::LogicalBlockWrite(LogicalBlockError {
                logical_block_id: self.id.clone(),
                description: "todo!()".to_string(),
            })),
        }
    }

    fn read_chunk_from_logical_block(
        &mut self,
        chunk_buffer: &mut [u8],
    ) -> Result<usize, UpdateError> {
        match self.source.file.read(chunk_buffer) {
            Ok(n) => Ok(n),
            Err(_) => Err(UpdateError::LogicalBlockRead(LogicalBlockError {
                logical_block_id: self.id.clone(),
                description: "todo!()".to_string(),
            })),
        }
    }

    fn write_chunk_in_file(
        &mut self,
        chunk_buffer: &mut [u8],
        file: &mut File,
    ) -> Result<usize, UpdateError> {
        match file.write(chunk_buffer) {
            Ok(n) => Ok(n),
            Err(_) => Err(UpdateError::LogicalBlockWrite(LogicalBlockError {
                logical_block_id: self.id.clone(),
                description: "todo!()".to_string(),
            })),
        }
    }

    pub(crate) fn verify(&self) -> Result<bool, UpdateError> {
        let public_key = self.get_public_key()?;
        let mut verifier = self.get_verifier(&public_key)?;

        self.update_verifier_with_logical_block_content(&mut verifier)?;

        let decoded_signature = general_purpose::STANDARD.decode(&self.signature).unwrap();

        match verifier.verify(&decoded_signature) {
            Ok(n) => Ok(n),
            Err(_) => Err(UpdateError::VerificationError(LogicalBlockError {
                logical_block_id: self.id.clone(),
                description: "todo!()".to_string(),
            })),
        }
    }

    fn get_public_key(&self) -> Result<PKey<Public>, UpdateError> {
        let mut public_key = Vec::new();
        File::open("./resources/test/test_public_key.pem")
            .unwrap()
            .read_to_end(&mut public_key)
            .unwrap();
        Ok(PKey::public_key_from_pem(&public_key).unwrap())
    }

    fn get_verifier(&'a self, public_key: &'a PKey<Public>) -> Result<Verifier<'a>, UpdateError> {
        let mut verifier = Verifier::new(MessageDigest::sha256(), public_key).unwrap();

        verifier.set_rsa_padding(Padding::PKCS1_PSS).unwrap();
        verifier
            .set_rsa_pss_saltlen(RsaPssSaltlen::custom(0))
            .unwrap();

        verifier.set_rsa_mgf1_md(MessageDigest::sha256()).unwrap();
        Ok(verifier)
    }

    fn update_verifier_with_logical_block_content(
        &self,
        verifier: &mut Verifier<'_>,
    ) -> Result<(), UpdateError> {
        let mut file = File::open(&self.destination.get_path()).unwrap();
        file.seek(std::io::SeekFrom::Start(self.destination.get_offset()))
            .unwrap();

        const CHUNK_SIZE: usize = 4096;
        let mut read_buffer = [0; CHUNK_SIZE];

        let total_bytes_to_read = self.destination.get_size();
        let mut total_bytes_read = 0;

        loop {
            let remaining_bytes = total_bytes_to_read - total_bytes_read;

            if remaining_bytes == 0 {
                return Ok(());
            }

            let bytes_to_read = if remaining_bytes >= CHUNK_SIZE {
                CHUNK_SIZE
            } else {
                remaining_bytes
            };

            match file.read_exact(&mut read_buffer[..bytes_to_read]) {
                Ok(_) => {
                    verifier.update(&read_buffer[..bytes_to_read]).unwrap();
                    total_bytes_read += bytes_to_read;
                }
                Err(_) => {
                    return Err(UpdateError::LogicalBlockRead(LogicalBlockError {
                        logical_block_id: self.id.clone(),
                        description: "todo!()".to_string(),
                    }))
                }
            }
        }
    }
}

impl<'a> fmt::Display for LogicalBlock<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} logical block (id: 0x{}, signature: {})",
            self.name, self.id, self.signature
        )
    }
}
