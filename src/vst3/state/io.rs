use super::{StreamError, VersionedPayload};
use crate::state::{HEADER_SIZE, StateHeader};
use toybox_vst3_ffi::ComRef;
use toybox_vst3_ffi::Steinberg::{IBStream, IBStreamTrait, int32, kResultOk};

/// Write all bytes to a VST3 `IBStream`.
///
/// # Safety
///
/// `stream` must be a valid VST3 stream pointer.
pub unsafe fn write_all(stream: *mut IBStream, bytes: &[u8]) -> Result<(), StreamError> {
    let Some(stream) = (unsafe { ComRef::from_raw(stream) }) else {
        return Err(StreamError::NullStream);
    };

    let mut written_total = 0usize;
    while written_total < bytes.len() {
        let remaining = &bytes[written_total..];
        let chunk = remaining.len().min(int32::MAX as usize) as int32;
        let mut written: int32 = 0;
        let result =
            unsafe { stream.write(remaining.as_ptr().cast_mut().cast(), chunk, &mut written) };
        if result != kResultOk || written > chunk {
            return Err(StreamError::IoFailure);
        }
        if written <= 0 {
            return Err(StreamError::UnexpectedEof);
        }
        written_total += written as usize;
    }

    Ok(())
}

/// Read exactly `bytes.len()` bytes from a VST3 `IBStream`.
///
/// # Safety
///
/// `stream` must be a valid VST3 stream pointer.
pub unsafe fn read_exact(stream: *mut IBStream, bytes: &mut [u8]) -> Result<(), StreamError> {
    let Some(stream) = (unsafe { ComRef::from_raw(stream) }) else {
        return Err(StreamError::NullStream);
    };

    let mut read_total = 0usize;
    while read_total < bytes.len() {
        let remaining = &mut bytes[read_total..];
        let chunk = remaining.len().min(int32::MAX as usize) as int32;
        let mut read: int32 = 0;
        let result = unsafe { stream.read(remaining.as_mut_ptr().cast(), chunk, &mut read) };
        if result != kResultOk || read > chunk {
            return Err(StreamError::IoFailure);
        }
        if read <= 0 {
            return Err(StreamError::UnexpectedEof);
        }
        read_total += read as usize;
    }

    Ok(())
}

/// Write a versioned payload to a VST3 stream.
///
/// # Safety
///
/// `stream` must be a valid VST3 stream pointer.
pub unsafe fn write_versioned_payload(
    stream: *mut IBStream,
    magic: u32,
    version: u32,
    payload: &[u8],
) -> Result<(), StreamError> {
    let bytes = super::try_encode_versioned_payload(magic, version, payload)?;
    // SAFETY: caller guarantees `stream` points to a valid IBStream.
    unsafe { write_all(stream, &bytes) }
}

/// Read and decode a versioned payload from a VST3 stream.
///
/// # Safety
///
/// `stream` must be a valid VST3 stream pointer.
pub unsafe fn read_versioned_payload(
    stream: *mut IBStream,
    expected_magic: u32,
    accepted_versions: &[u32],
) -> Result<VersionedPayload, StreamError> {
    let mut header = [0; HEADER_SIZE];
    // SAFETY: caller guarantees `stream` points to a valid IBStream.
    unsafe { read_exact(stream, &mut header) }?;

    let StateHeader {
        version,
        payload_len,
    } = StateHeader::decode(&header, expected_magic, accepted_versions)?;

    let mut payload = vec![0u8; payload_len];
    // SAFETY: caller guarantees `stream` points to a valid IBStream.
    unsafe { read_exact(stream, &mut payload) }?;

    Ok(VersionedPayload { version, payload })
}
