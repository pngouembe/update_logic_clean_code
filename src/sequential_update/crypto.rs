use std::{
    fs::File,
    io::{Read, Seek},
};

use base64::{engine::general_purpose, Engine};
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Public},
    rsa::Padding,
    sign::{RsaPssSaltlen, Verifier},
};

use crate::{
    reporting::{LogicalBlockError, UpdateError},
    sequential_update::memory::LogicalBlockDestination,
    sequential_update::software_archive::LogicalBlockInfo,
};

pub struct LogicalBlockVerifier {
    logical_block: LogicalBlockDestination,
    logical_block_info: LogicalBlockInfo,
    public_key: PKey<Public>,
}

impl LogicalBlockVerifier {
    pub fn from(
        logical_block_location: LogicalBlockDestination,
        logical_block_info: LogicalBlockInfo,
    ) -> LogicalBlockVerifier {
        let public_key = LogicalBlockVerifier::get_public_key();

        LogicalBlockVerifier {
            logical_block: logical_block_location,
            logical_block_info,
            public_key,
        }
    }

    fn get_public_key() -> PKey<Public> {
        let mut public_key = Vec::new();
        File::open("./resources/test/test_public_key.pem")
            .unwrap()
            .read_to_end(&mut public_key)
            .unwrap();
        PKey::public_key_from_pem(&public_key).unwrap()
    }

    pub(crate) fn verify(&self) -> Result<bool, UpdateError> {
        let mut verifier = self.get_verifier();

        let logical_block_file = self.get_logical_block_file();

        self.update_verifier_with_logical_block_content(&mut verifier, logical_block_file)?;

        let decoded_signature = general_purpose::STANDARD
            .decode(self.logical_block_info.get_signature())
            .unwrap();

        match verifier.verify(&decoded_signature) {
            Ok(n) => Ok(n),
            Err(_) => Err(UpdateError::VerificationError(LogicalBlockError {
                logical_block_id: self.logical_block_info.get_id(),
                description: "todo!()".to_string(),
            })),
        }
    }

    fn get_verifier(&self) -> Verifier<'_> {
        let mut verifier = Verifier::new(MessageDigest::sha256(), &self.public_key).unwrap();

        verifier.set_rsa_padding(Padding::PKCS1_PSS).unwrap();
        verifier
            .set_rsa_pss_saltlen(RsaPssSaltlen::custom(0))
            .unwrap();

        verifier.set_rsa_mgf1_md(MessageDigest::sha256()).unwrap();
        verifier
    }

    fn get_logical_block_file(&self) -> File {
        let mut logical_block_file = File::open(self.logical_block.get_path()).unwrap();
        logical_block_file
            .seek(std::io::SeekFrom::Start(self.logical_block.get_offset()))
            .unwrap();
        logical_block_file
    }

    fn update_verifier_with_logical_block_content(
        &self,
        verifier: &mut Verifier<'_>,
        mut logical_block_file: File,
    ) -> Result<(), UpdateError> {
        const CHUNK_SIZE: usize = 4096;
        let mut read_buffer = [0; CHUNK_SIZE];

        let total_bytes_to_read = self.logical_block.get_size();
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

            match logical_block_file.read_exact(&mut read_buffer[..bytes_to_read]) {
                Ok(_) => {
                    verifier.update(&read_buffer[..bytes_to_read]).unwrap();
                    total_bytes_read += bytes_to_read;
                }
                Err(_) => {
                    return Err(UpdateError::LogicalBlockRead(LogicalBlockError {
                        logical_block_id: self.logical_block_info.get_id(),
                        description: "todo!()".to_string(),
                    }))
                }
            }
        }
    }
}
