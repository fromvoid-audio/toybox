//! Versioned CLAP state payload helpers.
//!
//! These helpers persist state payloads in a fixed little-endian layout:
//! `magic:u32`, `version:u32`, `payload_len:u32`, then raw payload bytes.

use std::io::{Read, Write};

use clack_plugin::plugin::PluginError;
use clack_plugin::stream::{InputStream, OutputStream};

use crate::state::{HEADER_SIZE, HeaderError, StateHeader};

/// Error returned when serialized payloads exceed the supported size.
const VERSIONED_STATE_PAYLOAD_TOO_LARGE_ERROR: &str = "Plugin state payload exceeds max size";
/// Error returned when payload header validation fails.
const VERSIONED_STATE_PAYLOAD_HEADER_ERROR: &str = "Invalid plugin state payload";

/// Decoded versioned state payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedStatePayload {
    /// Payload version from the serialized state header.
    pub version: u32,
    /// Raw payload bytes following the state header.
    pub payload: Vec<u8>,
}

/// Write a versioned state payload to a CLAP output stream.
///
/// # Errors
/// Returns an error if the stream write fails, the payload exceeds
/// [`MAX_STATE_PAYLOAD_BYTES`](super::MAX_STATE_PAYLOAD_BYTES), or the
/// payload length does not fit into a `u32` header.
pub fn write_versioned_payload(
    output: &mut OutputStream,
    magic: u32,
    version: u32,
    payload: &[u8],
) -> Result<(), PluginError> {
    let header = StateHeader::encode(magic, version, payload.len()).map_err(header_error)?;
    output.write_all(&header)?;
    output.write_all(payload)?;
    Ok(())
}

/// Read and validate a versioned state payload from a CLAP input stream.
///
/// The payload is accepted only when:
/// - `magic` matches `expected_magic`,
/// - `version` is listed in `supported_versions`,
/// - `payload_len` does not exceed [`MAX_STATE_PAYLOAD_BYTES`](super::MAX_STATE_PAYLOAD_BYTES).
///
/// # Errors
/// Returns a descriptive [`PluginError::Message`] when validation fails, or an
/// I/O derived error if the stream read fails.
pub fn read_versioned_payload(
    input: &mut InputStream,
    expected_magic: u32,
    supported_versions: &[u32],
) -> Result<VersionedStatePayload, PluginError> {
    let mut bytes = [0; HEADER_SIZE];
    input.read_exact(&mut bytes)?;
    let StateHeader {
        version,
        payload_len,
    } = StateHeader::decode(&bytes, expected_magic, supported_versions).map_err(header_error)?;

    let mut payload = vec![0u8; payload_len];
    input.read_exact(&mut payload)?;

    Ok(VersionedStatePayload { version, payload })
}

/// Preserve the CLAP error messages for malformed and oversized state.
fn header_error(error: HeaderError) -> PluginError {
    PluginError::Message(match error {
        HeaderError::InvalidMagic | HeaderError::UnsupportedVersion => {
            VERSIONED_STATE_PAYLOAD_HEADER_ERROR
        }
        HeaderError::PayloadTooLarge => VERSIONED_STATE_PAYLOAD_TOO_LARGE_ERROR,
    })
}

#[cfg(test)]
mod tests;
