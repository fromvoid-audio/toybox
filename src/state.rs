//! Shared binary header for versioned plugin state.

/// Maximum accepted versioned state payload size in bytes.
///
/// This constant is shared by CLAP and VST3 state helpers to keep state size
/// limits consistent across host formats.
pub const MAX_STATE_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;

/// Length of the little-endian magic, version and payload length fields.
pub(crate) const HEADER_SIZE: usize = 12;

/// Validated fields used by the format-specific readers.
pub(crate) struct StateHeader {
    /// Version reported by the state writer.
    pub(crate) version: u32,
    /// Exact number of payload bytes following the header.
    pub(crate) payload_len: usize,
}

/// Header errors that each host wrapper maps to its public error type.
#[derive(Debug)]
pub(crate) enum HeaderError {
    /// The format marker did not match the caller's expectation.
    InvalidMagic,
    /// The caller does not support the stored version.
    UnsupportedVersion,
    /// The stored or requested size exceeds the state limit.
    PayloadTooLarge,
}

impl StateHeader {
    /// Encode the header after checking that the payload length is supported.
    pub(crate) fn encode(
        magic: u32,
        version: u32,
        payload_len: usize,
    ) -> Result<[u8; HEADER_SIZE], HeaderError> {
        validate_payload_len(payload_len)?;
        let payload_len = u32::try_from(payload_len).map_err(|_| HeaderError::PayloadTooLarge)?;

        let mut bytes = [0; HEADER_SIZE];
        bytes[..4].copy_from_slice(&magic.to_le_bytes());
        bytes[4..8].copy_from_slice(&version.to_le_bytes());
        bytes[8..].copy_from_slice(&payload_len.to_le_bytes());
        Ok(bytes)
    }

    /// Validate the header before any payload bytes are allocated or read.
    pub(crate) fn decode(
        bytes: &[u8; HEADER_SIZE],
        expected_magic: u32,
        accepted_versions: &[u32],
    ) -> Result<Self, HeaderError> {
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic != expected_magic {
            return Err(HeaderError::InvalidMagic);
        }
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if !accepted_versions.contains(&version) {
            return Err(HeaderError::UnsupportedVersion);
        }
        let payload_len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        validate_payload_len(payload_len)?;

        Ok(Self {
            version,
            payload_len,
        })
    }
}

/// Limit state allocations before any format reads or writes the payload.
fn validate_payload_len(payload_len: usize) -> Result<(), HeaderError> {
    if payload_len > MAX_STATE_PAYLOAD_BYTES {
        return Err(HeaderError::PayloadTooLarge);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_accepts_exact_payload_limit() {
        let bytes = StateHeader::encode(1, 2, MAX_STATE_PAYLOAD_BYTES)
            .expect("size limit fits in the header");
        let header =
            StateHeader::decode(&bytes, 1, &[2]).expect("size limit is accepted when reading");
        assert_eq!(header.payload_len, MAX_STATE_PAYLOAD_BYTES);
    }
}
