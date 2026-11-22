//! State serialization helpers for VST3 plugins.

use std::fmt;

use crate::state::HeaderError;
pub use crate::state::MAX_STATE_PAYLOAD_BYTES;

/// Internal codec helpers for VST3 payload encoding and decoding.
mod codec;
/// Internal stream adapters for reading/writing versioned VST3 state payloads.
mod io;

pub use codec::{decode_versioned_payload, try_encode_versioned_payload};
pub use io::{read_exact, read_versioned_payload, write_all, write_versioned_payload};

/// Serialized state payload read from a versioned stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionedPayload {
    /// Version stored in the serialized payload header.
    pub version: u32,
    /// Opaque payload bytes.
    pub payload: Vec<u8>,
}

/// Error returned by VST3 state stream helpers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamError {
    /// The host supplied a null stream pointer.
    NullStream,
    /// The VST3 stream method failed or reported more bytes than requested.
    IoFailure,
    /// The stream ended before all requested bytes were read.
    UnexpectedEof,
    /// The payload magic did not match the expected value.
    InvalidMagic,
    /// The payload version is not supported.
    UnsupportedVersion,
    /// The serialized state header is malformed.
    InvalidHeader,
    /// The payload length exceeds `MAX_STATE_PAYLOAD_BYTES`.
    PayloadTooLarge,
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamError::NullStream => write!(f, "state stream pointer is null"),
            StreamError::IoFailure => write!(f, "state stream I/O failed"),
            StreamError::UnexpectedEof => write!(f, "state stream ended unexpectedly"),
            StreamError::InvalidMagic => write!(f, "state payload magic mismatch"),
            StreamError::UnsupportedVersion => write!(f, "state payload version is unsupported"),
            StreamError::InvalidHeader => write!(f, "state payload header is invalid"),
            StreamError::PayloadTooLarge => {
                write!(f, "state payload exceeds max supported size")
            }
        }
    }
}

impl std::error::Error for StreamError {}

impl From<HeaderError> for StreamError {
    fn from(error: HeaderError) -> Self {
        match error {
            HeaderError::InvalidMagic => Self::InvalidMagic,
            HeaderError::UnsupportedVersion => Self::UnsupportedVersion,
            HeaderError::PayloadTooLarge => Self::PayloadTooLarge,
        }
    }
}

/// Compatibility entrypoint for writing a versioned payload.
///
/// Prefer [`try_encode_versioned_payload`] for fallible handling.
///
/// # Panics
///
/// Panics if the payload exceeds [`MAX_STATE_PAYLOAD_BYTES`].
pub fn encode_versioned_payload(magic: u32, version: u32, payload: &[u8]) -> Vec<u8> {
    let Ok(bytes) = try_encode_versioned_payload(magic, version, payload) else {
        panic!("VST3 state payload is too large");
    };

    bytes
}

#[cfg(test)]
mod tests {
    mod codec_tests;
    mod io_helpers;
    mod io_tests;
}
