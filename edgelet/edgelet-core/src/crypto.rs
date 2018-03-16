// Copyright (c) Microsoft. All rights reserved.

// TODO remove me when warnings are cleaned up
#![allow(warnings)]
use std::collections::HashMap;

use bytes::Bytes;
use failure::ResultExt;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use error::{Error, ErrorKind};

pub trait Sign {
    fn sign(
        &self,
        signature_algorithm: SignatureAlgorithm,
        data: &[u8],
    ) -> Result<Signature, Error>;
}

pub enum SignatureAlgorithm {
    HMACSHA256,
}

// TODO add cryptographically secure equal
#[derive(Debug, PartialEq)]
pub struct Signature {
    bytes: Bytes,
}

impl Signature {
    pub fn new(bytes: Bytes) -> Signature {
        Signature { bytes }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

pub struct InMemoryKey {
    key: Bytes,
}

impl Sign for InMemoryKey {
    fn sign(
        &self,
        signature_algorithm: SignatureAlgorithm,
        data: &[u8],
    ) -> Result<Signature, Error> {
        let signature = match signature_algorithm {
            SignatureAlgorithm::HMACSHA256 => {
                // Create `Mac` trait implementation, namely HMAC-SHA256
                let mut mac =
                    Hmac::<Sha256>::new(&self.key).map_err(|_| ErrorKind::Sign(self.key.len()))?;
                mac.input(data);

                // `result` has type `MacResult` which is a thin wrapper around array of
                // bytes for providing constant time equality check
                let result = mac.result();
                // To get underlying array use `code` method, but be careful, since
                // incorrect use of the code value may permit timing attacks which defeat
                // the security provided by the `MacResult` (https://docs.rs/hmac/0.5.0/hmac/)
                let code_bytes = result.code();

                Signature::new(Bytes::from(code_bytes.as_ref()))
            }
        };
        Ok(signature)
    }
}

pub trait KeyStore {
    type Key: Sign;

    fn get(&self, identity: &str, key_name: &str) -> Option<&Self::Key>;
}

pub struct MemoryKeyStore {
    keys: HashMap<String, InMemoryKey>,
}

impl MemoryKeyStore {
    pub fn new() -> MemoryKeyStore {
        MemoryKeyStore {
            keys: HashMap::new(),
        }
    }
}

impl KeyStore for MemoryKeyStore {
    type Key = InMemoryKey;

    fn get(&self, identity: &str, key_name: &str) -> Option<&Self::Key> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use crypto::{InMemoryKey, Sign, SignatureAlgorithm};

    #[test]
    fn sha256_sign_test_positive() {
        //Arrange
        let in_memory_key = InMemoryKey {
            key: Bytes::from("key"),
        };
        let data = b"The quick brown fox jumps over the lazy dog";
        let signature_algorithm = SignatureAlgorithm::HMACSHA256;
        //Act
        let result_hmac256 = in_memory_key.sign(signature_algorithm, data).unwrap();

        //Assert
        let expected = [
            0xf7, 0xbc, 0x83, 0xf4, 0x30, 0x53, 0x84, 0x24, 0xb1, 0x32, 0x98, 0xe6, 0xaa, 0x6f,
            0xb1, 0x43, 0xef, 0x4d, 0x59, 0xa1, 0x49, 0x46, 0x17, 0x59, 0x97, 0x47, 0x9d, 0xbc,
            0x2d, 0x1a, 0x3c, 0xd8,
        ];
        assert_eq!(expected, result_hmac256.as_bytes());
    }

    #[test]
    #[should_panic]
    fn sha256_sign_test_data_not_mathing_shall_fail() {
        //Arrange
        let in_memory_key = InMemoryKey {
            key: Bytes::from("key"),
        };
        let data = b"The quick brown fox jumps over the lazy do";
        let signature_algorithm = SignatureAlgorithm::HMACSHA256;
        //Act
        let result_hmac256 = in_memory_key.sign(signature_algorithm, data).unwrap();

        //Assert
        let expected = [
            0xf7, 0xbc, 0x83, 0xf4, 0x30, 0x53, 0x84, 0x24, 0xb1, 0x32, 0x98, 0xe6, 0xaa, 0x6f,
            0xb1, 0x43, 0xef, 0x4d, 0x59, 0xa1, 0x49, 0x46, 0x17, 0x59, 0x97, 0x47, 0x9d, 0xbc,
            0x2d, 0x1a, 0x3c, 0xd8,
        ];
        assert_eq!(expected, result_hmac256.as_bytes());
    }

    #[test]
    #[should_panic]
    fn sha256_sign_test_key_not_mathing_shall_fail() {
        //Arrange
        let in_memory_key = InMemoryKey {
            key: Bytes::from("wrongkey"),
        };
        let data = b"The quick brown fox jumps over the lazy dog";
        let signature_algorithm = SignatureAlgorithm::HMACSHA256;
        //Act
        let result_hmac256 = in_memory_key.sign(signature_algorithm, data).unwrap();

        //Assert
        let expected = [
            0xf7, 0xbc, 0x83, 0xf4, 0x30, 0x53, 0x84, 0x24, 0xb1, 0x32, 0x98, 0xe6, 0xaa, 0x6f,
            0xb1, 0x43, 0xef, 0x4d, 0x59, 0xa1, 0x49, 0x46, 0x17, 0x59, 0x97, 0x47, 0x9d, 0xbc,
            0x2d, 0x1a, 0x3c, 0xd8,
        ];
        assert_eq!(expected, result_hmac256.as_bytes());
    }
}
