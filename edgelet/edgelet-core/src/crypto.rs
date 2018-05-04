// Copyright (c) Microsoft. All rights reserved.

use std::collections::HashMap;
use std::convert::AsRef;
use std::sync::{Arc, RwLock};

use bytes::Bytes;
use consistenttime::ct_u8_slice_eq;
use failure::ResultExt;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use error::{Error, ErrorKind};
use certificate_properties::CertificateProperties;

pub trait Sign {
    type Signature: Signature;

    fn sign(
        &self,
        signature_algorithm: SignatureAlgorithm,
        data: &[u8],
    ) -> Result<Self::Signature, Error>;
}

pub trait KeyStore {
    type Key: Sign;

    fn get(&self, identity: &str, key_name: &str) -> Result<Self::Key, Error>;
}

pub enum SignatureAlgorithm {
    HMACSHA256,
}

pub trait Signature {
    fn as_bytes(&self) -> &[u8];
}

impl<T> Signature for T
where
    T: AsRef<[u8]>,
{
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

pub enum PrivateKey<T: AsRef<[u8]>> {
    Ref(String),
    Key(T),
}

pub trait CreateCertificate {
    type Certificate: Certificate;

    fn create_certificate(
        &self,
        properties: &CertificateProperties,
    ) -> Result<Self::Certificate, Error>;
}

pub trait Certificate {
    type Buffer: AsRef<[u8]>;
    type KeyBuffer: AsRef<[u8]>;

    fn pem(&self) -> Result<Self::Buffer, Error>;
    fn get_private_key(&self) -> Result<(u32, PrivateKey<Self::KeyBuffer>), Error>;
}

pub trait GetTrustBundle {
    type Certificate: Certificate;

    fn get_trust_bundle(&self) -> Result<Self::Certificate, Error>;
}

pub trait MakeRandom {
    fn get_random_bytes(&self, buffer: &mut [u8]) -> Result<(), Error>;
}

pub trait Encrypt {
    type Buffer: AsRef<[u8]>;

    fn encrypt(
        &self,
        client_id: &[u8],
        plaintext: &[u8],
        passphrase: Option<&[u8]>,
        initialization_vector: &[u8],
    ) -> Result<Self::Buffer, Error>;
}

pub trait Decrypt {
    type Buffer: AsRef<[u8]>;

    fn decrypt(
        &self,
        client_id: &[u8],
        ciphertext: &[u8],
        passphrase: Option<&[u8]>,
        initialization_vector: &[u8],
    ) -> Result<Self::Buffer, Error>;
}

#[derive(Debug)]
pub struct Digest {
    bytes: Bytes,
}

impl PartialEq for Digest {
    fn eq(&self, other: &Digest) -> bool {
        ct_u8_slice_eq(self.bytes.as_ref(), other.bytes.as_ref())
    }
}

impl Signature for Digest {
    fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl Digest {
    pub fn new(bytes: Bytes) -> Digest {
        Digest { bytes }
    }
}

#[derive(Clone, Debug)]
pub struct MemoryKey {
    key: Bytes,
}

impl MemoryKey {
    pub fn new<B: AsRef<[u8]>>(key: B) -> MemoryKey {
        MemoryKey {
            key: Bytes::from(key.as_ref()),
        }
    }
}

impl Sign for MemoryKey {
    type Signature = Digest;

    fn sign(
        &self,
        signature_algorithm: SignatureAlgorithm,
        data: &[u8],
    ) -> Result<Self::Signature, Error> {
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

                Digest::new(Bytes::from(code_bytes.as_ref()))
            }
        };
        Ok(signature)
    }
}

impl AsRef<[u8]> for MemoryKey {
    fn as_ref(&self) -> &[u8] {
        &self.key
    }
}

#[derive(Clone, Default)]
pub struct MemoryKeyStore {
    keys: Arc<RwLock<HashMap<String, MemoryKey>>>,
}

impl MemoryKeyStore {
    pub fn new() -> MemoryKeyStore {
        MemoryKeyStore {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Inserts a key-value pair into the KeyStore.
    ///
    /// If the store did not have this key present, None is returned.
    ///
    /// If the store did have this key (by Identity and Key_name) present, the value (Key) is updated and the old value is returned.
    pub fn insert(
        &mut self,
        identity: &str,
        key_name: &str,
        key_value: MemoryKey,
    ) -> Option<MemoryKey> {
        self.keys
            .write()
            .expect("Failed to acquire a write lock")
            .insert(format!("{}{}", identity, key_name), key_value)
    }

    pub fn is_empty(&self) -> bool {
        self.keys
            .read()
            .expect("Failed to acquire a read lock")
            .is_empty()
    }

    pub fn len(&self) -> usize {
        self.keys
            .read()
            .expect("Failed to acquire a read lock")
            .len()
    }
}

impl KeyStore for MemoryKeyStore {
    type Key = MemoryKey;

    fn get(&self, identity: &str, key_name: &str) -> Result<Self::Key, Error> {
        self.keys
            .read()
            .expect("Failed to acquire a read lock")
            .get(&format!("{}{}", identity, key_name))
            .cloned()
            .ok_or_else(|| Error::from(ErrorKind::NotFound))
    }
}

#[derive(Clone)]
pub struct DerivedKeyStore<K> {
    root: Arc<K>,
}

impl<K> DerivedKeyStore<K> {
    pub fn new(root: K) -> Self {
        DerivedKeyStore {
            root: Arc::new(root),
        }
    }
}

impl<K: Sign> KeyStore for DerivedKeyStore<K> {
    type Key = MemoryKey;

    fn get(&self, identity: &str, key_name: &str) -> Result<Self::Key, Error> {
        self.root
            .sign(
                SignatureAlgorithm::HMACSHA256,
                format!("{}{}", identity, key_name).as_bytes(),
            )
            .map(|d| MemoryKey::new(d.as_bytes()))
            .context(ErrorKind::KeyStore)
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64;
    use bytes::Bytes;

    #[test]
    fn sha256_sign_test_positive() {
        //Arrange
        let in_memory_key = MemoryKey {
            key: Bytes::from("key"),
        };
        let data = b"The quick brown fox jumps over the lazy dog";
        let signature_algorithm = SignatureAlgorithm::HMACSHA256;
        //Act
        let result_hmac256 = in_memory_key.sign(signature_algorithm, data).unwrap();

        //Assert
        let expected_bytes = [
            0xf7, 0xbc, 0x83, 0xf4, 0x30, 0x53, 0x84, 0x24, 0xb1, 0x32, 0x98, 0xe6, 0xaa, 0x6f,
            0xb1, 0x43, 0xef, 0x4d, 0x59, 0xa1, 0x49, 0x46, 0x17, 0x59, 0x97, 0x47, 0x9d, 0xbc,
            0x2d, 0x1a, 0x3c, 0xd8,
        ];
        let expected_signature = Digest::new(Bytes::from(expected_bytes.as_ref()));

        assert_eq!(expected_bytes, result_hmac256.as_bytes());
        assert_eq!(expected_signature, result_hmac256);
    }

    #[test]
    fn sha256_sign_test_data_not_matching_shall_fail() {
        //Arrange
        let in_memory_key = MemoryKey {
            key: Bytes::from("key"),
        };
        let data = b"The quick brown fox jumps over the lazy do";
        let signature_algorithm = SignatureAlgorithm::HMACSHA256;
        //Act
        let result_hmac256 = in_memory_key.sign(signature_algorithm, data).unwrap();

        //Assert
        let expected_bytes = [
            0xf7, 0xbc, 0x83, 0xf4, 0x30, 0x53, 0x84, 0x24, 0xb1, 0x32, 0x98, 0xe6, 0xaa, 0x6f,
            0xb1, 0x43, 0xef, 0x4d, 0x59, 0xa1, 0x49, 0x46, 0x17, 0x59, 0x97, 0x47, 0x9d, 0xbc,
            0x2d, 0x1a, 0x3c, 0xd8,
        ];

        let expected_signature = Digest::new(Bytes::from(expected_bytes.as_ref()));

        assert_ne!(expected_signature, result_hmac256);
    }

    #[test]
    fn sha256_sign_test_key_not_mathing_shall_fail() {
        //Arrange
        let in_memory_key = MemoryKey {
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

        assert_ne!(expected, result_hmac256.as_bytes());
    }

    //MemoryKeyStoreTests
    #[test]
    fn create_empty_memory_keystore() {
        //Arrange
        //Act
        let memory_key_store = MemoryKeyStore::new();

        //Assert
        assert_eq!(true, memory_key_store.is_empty());
    }

    #[test]
    fn create_memory_keystore_1key() {
        //Arrange
        let mut memory_key_store = MemoryKeyStore::new();
        let in_memory_key = MemoryKey {
            key: Bytes::from("anykey"),
        };

        //Act
        memory_key_store.insert("mod1", "key1", in_memory_key);

        //Assert
        assert_eq!(false, memory_key_store.is_empty());
        assert_eq!(false, memory_key_store.get("mod1", "invalidKey").is_ok());
        assert_eq!(true, memory_key_store.get("mod1", "key1").is_ok());
    }

    #[test]
    fn create_memory_keystore_2keys() {
        //Arrange
        let mut memory_key_store = MemoryKeyStore::new();
        let in_memory_key = MemoryKey {
            key: Bytes::from("anykey"),
        };

        let in_memory_key2 = MemoryKey {
            key: Bytes::from("anykey"),
        };

        //Act
        memory_key_store.insert("mod1", "key1", in_memory_key);
        memory_key_store.insert("mod2", "key2", in_memory_key2);

        //Assert
        assert_eq!(false, memory_key_store.is_empty());
        assert_eq!(false, memory_key_store.get("mod1", "invalidKey").is_ok());
        assert_eq!(true, memory_key_store.get("mod1", "key1").is_ok());
        assert_eq!(true, memory_key_store.get("mod2", "key2").is_ok());
        assert_eq!(2, memory_key_store.len());
    }

    #[test]
    fn derived_key_store() {
        let key_store = DerivedKeyStore::new(MemoryKey::new("key"));
        let key = key_store.get("key2", "primary").unwrap();
        let digest = key.sign(
            SignatureAlgorithm::HMACSHA256,
            b"The quick brown fox jumps over the lazy dog",
        ).unwrap();
        assert_eq!(
            "wBXO109hMjTfjUtQGtTmeqiqoqboLl8F5b7tR0of5yE=",
            base64::encode(digest.as_bytes())
        );
    }
}
