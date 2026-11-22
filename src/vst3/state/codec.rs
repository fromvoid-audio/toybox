use super::{StreamError, VersionedPayload};
use crate::state::{HEADER_SIZE, StateHeader};

/// Encode a versioned state payload into a byte vector.
///
/// Returns [`StreamError::PayloadTooLarge`] when payload length exceeds
/// the configured payload limit.
pub fn try_encode_versioned_payload(
    magic: u32,
    version: u32,
    payload: &[u8],
) -> Result<Vec<u8>, StreamError> {
    let header = StateHeader::encode(magic, version, payload.len())?;
    let mut bytes = Vec::with_capacity(HEADER_SIZE + payload.len());
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

/// Decode a versioned state payload from bytes.
pub fn decode_versioned_payload(
    bytes: &[u8],
    expected_magic: u32,
    accepted_versions: &[u32],
) -> Result<VersionedPayload, StreamError> {
    let (header, payload) = bytes
        .split_first_chunk::<HEADER_SIZE>()
        .ok_or(StreamError::InvalidHeader)?;
    let StateHeader {
        version,
        payload_len,
    } = StateHeader::decode(header, expected_magic, accepted_versions)?;
    if payload.len() != payload_len {
        return Err(StreamError::InvalidHeader);
    }

    Ok(VersionedPayload {
        version,
        payload: payload.to_vec(),
    })
}
