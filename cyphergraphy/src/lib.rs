// Set of libraries for privacy-preserving networking apps
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2019-2023 by
//     Dr. Maxim Orlovsky <orlovsky@cyphernet.org>
//
// Copyright 2022-2023 Cyphernet Initiative, Switzerland
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Implementation-independent abstractions for main cryptographic algorithms
//! used for end-to-end encryption and authorization.

#[macro_use]
extern crate amplify;

mod digest;
#[cfg(feature = "x25519")]
pub mod x25519;
#[cfg(feature = "ed25519")]
pub mod ed25519;

pub use digest::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display("invalid secret key")]
#[non_exhaustive]
pub struct EcSkInvalid {}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display("invalid public key")]
#[non_exhaustive]
pub struct EcPkInvalid {}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
#[non_exhaustive]
pub enum EcdhError {
    /// public key provided for the ECDH is weak key
    WeakPk,

    #[display(inner)]
    #[from]
    InvalidPk(EcPkInvalid),

    #[display(inner)]
    #[from]
    InvalidSk(EcSkInvalid),
}

#[derive(Clone, Eq, PartialEq, Debug, Display, Error, From)]
#[display(doc_comments)]
pub enum EcSerError {
    #[display(inner)]
    #[from]
    Io(amplify::IoError),

    /// public key has invalid length {0}
    InvalidKeyLength(usize),

    #[display(inner)]
    #[from]
    InvalidKey(EcPkInvalid),

    /// error parsing encoding: {0}
    DataEncoding(String),
}

/// Elliptic-curve based public key type which can be used in ECDH or signature schemes.
///
/// # Safety
///
/// The type provides no guarantees on the key validity upon deserialization.
pub trait EcPk: Clone + Eq {
    const COMPRESSED_LEN: usize;
    const CURVE_NAME: &'static str;

    // TODO: When generic_const_exprs arrive switch to Self::COMPRESSED_LEN arrays
    type Compressed: Copy + Sized + Send + AsRef<[u8]>;

    fn base_point() -> Self;

    fn to_pk_compressed(&self) -> Self::Compressed;
    fn from_pk_compressed(pk: Self::Compressed) -> Result<Self, EcPkInvalid>;
    fn from_pk_compressed_slice(slice: &[u8]) -> Result<Self, EcPkInvalid>;
}

/// Elliptic-curve based private key type.
///
/// # Safety
///
/// The type provides no guarantees on the key validity upon deserialization.
pub trait EcSk: Eq {
    type Pk: EcPk;

    fn generate_keypair() -> (Self, Self::Pk)
    where Self: Sized;
    fn to_pk(&self) -> Result<Self::Pk, EcSkInvalid>;
}

/// Elliptic-curve based public key type which can be used for ECDH.
///
/// # Safety
///
/// The type provides no guarantees on the key validity upon deserialization.
pub trait Ecdh: EcSk {
    type SharedSecret: Copy + Eq + Sized + Send + AsRef<[u8]>;

    fn ecdh(&self, pk: &Self::Pk) -> Result<Self::SharedSecret, EcdhError>;
}

/// Signature scheme trait
pub trait EcSign: EcSk {
    type Sig: Copy + Sized + Send;

    fn sign(&self, msg: impl AsRef<[u8]>) -> Self::Sig;
}

mod ed22519_compact_err_convert {
    use ed25519_compact::Error;

    use super::*;

    impl From<Error> for EcPkInvalid {
        fn from(err: Error) -> Self {
            match err {
                Error::InvalidPublicKey => EcPkInvalid {},

                Error::WeakPublicKey
                | Error::InvalidSecretKey
                | Error::SignatureMismatch
                | Error::InvalidSignature
                | Error::InvalidSeed
                | Error::InvalidBlind
                | Error::InvalidNoise
                | Error::ParseError
                | Error::NonCanonical => {
                    unreachable!("ECDH in ed25519-compact crate should not generate this errors")
                }
            }
        }
    }

    impl From<Error> for EcSkInvalid {
        fn from(err: Error) -> Self {
            match err {
                Error::InvalidSecretKey => EcSkInvalid {},

                Error::WeakPublicKey
                | Error::InvalidPublicKey
                | Error::SignatureMismatch
                | Error::InvalidSignature
                | Error::InvalidSeed
                | Error::InvalidBlind
                | Error::InvalidNoise
                | Error::ParseError
                | Error::NonCanonical => {
                    unreachable!("ECDH in ed25519-compact crate should not generate this errors")
                }
            }
        }
    }

    impl From<Error> for EcSerError {
        fn from(err: Error) -> Self { EcSerError::DataEncoding(err.to_string()) }
    }

    impl From<Error> for EcdhError {
        fn from(err: Error) -> Self {
            match err {
                Error::WeakPublicKey => EcdhError::WeakPk,
                Error::InvalidPublicKey => EcdhError::InvalidPk(EcPkInvalid {}),
                Error::InvalidSecretKey => EcdhError::InvalidSk(EcSkInvalid {}),

                Error::SignatureMismatch
                | Error::InvalidSignature
                | Error::InvalidSeed
                | Error::InvalidBlind
                | Error::InvalidNoise
                | Error::ParseError
                | Error::NonCanonical => {
                    unreachable!("ECDH in ed25519-compact crate should not generate this errors")
                }
            }
        }
    }
}

#[cfg(feature = "multibase")]
impl From<multibase::Error> for EcSerError {
    fn from(err: multibase::Error) -> Self { EcSerError::DataEncoding(err.to_string()) }
}