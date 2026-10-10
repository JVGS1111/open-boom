//! Audio sample format conversion to normalized 32-bit floating point PCM.

use open_boom_core::types::{EngineError, EngineResult};

/// Ingest audio sample formats supported by Windows WASAPI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    /// 32-bit IEEE 754 Floating Point (WASAPI standard shared mode default).
    Float32,
    /// 16-bit signed integer PCM.
    Pcm16,
    /// 24-bit signed integer PCM packed into 3 bytes.
    Pcm24Packed,
    /// 24-bit signed integer PCM padded in a 32-bit container (MSB aligned in bits 8..31).
    Pcm24In32,
    /// 32-bit signed integer PCM.
    Pcm32,
}

impl SampleFormat {
    /// Returns the number of bytes required per single sample in this format.
    #[inline(always)]
    pub const fn bytes_per_sample(&self) -> usize {
        match self {
            Self::Float32 => 4,
            Self::Pcm16 => 2,
            Self::Pcm24Packed => 3,
            Self::Pcm24In32 => 4,
            Self::Pcm32 => 4,
        }
    }
}

/// Converts a raw byte slice into normalized 32-bit float samples in `[-1.0, 1.0]`.
///
/// Guaranteed zero heap allocations in the active processing path.
pub fn convert_samples_to_f32(
    src_bytes: &[u8],
    format: SampleFormat,
    dst: &mut [f32],
) -> EngineResult<usize> {
    let bps = format.bytes_per_sample();
    if src_bytes.is_empty() {
        return Ok(0);
    }
    if src_bytes.len() % bps != 0 {
        return Err(EngineError::UnsupportedFormat(format!(
            "Input buffer length {} is not a multiple of sample size {} for {:?}",
            src_bytes.len(),
            bps,
            format
        )));
    }

    let sample_count = src_bytes.len() / bps;
    if dst.len() < sample_count {
        return Err(EngineError::BufferError(format!(
            "Destination buffer capacity {} too small for {} samples",
            dst.len(),
            sample_count
        )));
    }

    match format {
        SampleFormat::Float32 => {
            for (i, chunk) in src_bytes.chunks_exact(4).enumerate() {
                let bytes: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
                let sample = f32::from_le_bytes(bytes);
                dst[i] = sample.clamp(-1.0, 1.0);
            }
        }
        SampleFormat::Pcm16 => {
            for (i, chunk) in src_bytes.chunks_exact(2).enumerate() {
                let bytes: [u8; 2] = [chunk[0], chunk[1]];
                let raw = i16::from_le_bytes(bytes);
                // Signed 16-bit normalized to [-1.0, 1.0]
                let sample = raw as f32 / 32768.0;
                dst[i] = sample.clamp(-1.0, 1.0);
            }
        }
        SampleFormat::Pcm24Packed => {
            for (i, chunk) in src_bytes.chunks_exact(3).enumerate() {
                // Little-endian signed 24-bit with sign extension from byte 2
                let raw = (chunk[0] as i32)
                    | ((chunk[1] as i32) << 8)
                    | ((chunk[2] as i8 as i32) << 16);
                let sample = raw as f32 / 8388608.0;
                dst[i] = sample.clamp(-1.0, 1.0);
            }
        }
        SampleFormat::Pcm24In32 => {
            for (i, chunk) in src_bytes.chunks_exact(4).enumerate() {
                let bytes: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
                let raw_i32 = i32::from_le_bytes(bytes);
                // Windows WAVEFORMATEXTENSIBLE MSB-aligned 24-bit in 32-bit container
                let raw = raw_i32 >> 8;
                let sample = raw as f32 / 8388608.0;
                dst[i] = sample.clamp(-1.0, 1.0);
            }
        }
        SampleFormat::Pcm32 => {
            for (i, chunk) in src_bytes.chunks_exact(4).enumerate() {
                let bytes: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
                let raw = i32::from_le_bytes(bytes);
                let sample = raw as f32 / 2147483648.0;
                dst[i] = sample.clamp(-1.0, 1.0);
            }
        }
    }

    Ok(sample_count)
}
