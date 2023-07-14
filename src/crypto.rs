use std::{
    fs::File,
    io::{Read, Seek},
};

use openssl::{
    hash::MessageDigest,
    pkey::PKey,
    rsa::Padding,
    sign::{RsaPssSaltlen, Verifier},
};

use crate::{
    memory::LogicalBlockLocation,
    reporting::{LogicalBlockError, UpdateError},
    software_archive::LogicalBlockInfo,
};

pub struct LogicalBlockVerifier {
    logical_block: LogicalBlockLocation,
    logical_block_info: LogicalBlockInfo,
}

impl LogicalBlockVerifier {
    pub fn from(
        logical_block_location: LogicalBlockLocation,
        logical_block_info: LogicalBlockInfo,
    ) -> LogicalBlockVerifier {
        LogicalBlockVerifier {
            logical_block: logical_block_location,
            logical_block_info,
        }
    }

    pub(crate) fn verify(&self) -> Result<bool, UpdateError> {
        let mut public_key = Vec::new();
        File::open("./resources/test/test_public_key.pem")
            .unwrap()
            .read_to_end(&mut public_key)
            .unwrap();
        let public_key = PKey::public_key_from_pem(&public_key).unwrap();

        let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key).unwrap();

        verifier.set_rsa_padding(Padding::PKCS1_PSS).unwrap();
        verifier
            .set_rsa_pss_saltlen(RsaPssSaltlen::custom(0))
            .unwrap();

        verifier.set_rsa_mgf1_md(MessageDigest::sha256()).unwrap();

        let mut logical_block_file = File::open(self.logical_block.get_path()).unwrap();
        logical_block_file
            .seek(std::io::SeekFrom::Start(self.logical_block.get_offset()))
            .unwrap();

        const CHUNK_SIZE: usize = 4096;

        let mut read_buffer = [0; CHUNK_SIZE];

        let total_bytes_to_read = self.logical_block.get_size();
        let mut total_bytes_read = 0;

        loop {
            let remaining_bytes = total_bytes_to_read - total_bytes_read;

            if remaining_bytes == 0 {
                let decoded_signature = general_purpose::STANDARD
                    .decode(self.logical_block_info.get_signature())
                    .unwrap();

                match verifier.verify(&decoded_signature) {
                    Ok(n) => return Ok(n),
                    Err(_) => {
                        return Err(UpdateError::VerificationError(LogicalBlockError {
                            logical_block_id: self.logical_block_info.get_id(),
                            description: "todo!()".to_string(),
                        }));
                    }
                }
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
