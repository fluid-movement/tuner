//! Loading recordings as mono `f32` samples.

use std::path::Path;

/// A mono recording.
#[derive(Debug, Clone)]
pub struct Audio {
    pub samples: Vec<f32>,
    pub sample_rate: f32,
}

impl Audio {
    pub fn seconds(&self) -> f32 {
        self.samples.len() as f32 / self.sample_rate
    }
}

/// Reads a WAV or FLAC file (by extension), mixing all channels to mono and
/// scaling integer samples to -1..1.
pub fn load(path: &Path) -> Result<Audio, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "wav" => load_wav(path),
        "flac" => load_flac(path),
        _ => Err(format!("unsupported file type: {}", path.display())),
    }
}

fn load_wav(path: &Path) -> Result<Audio, String> {
    let mut reader =
        hound::WavReader::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let interleaved: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<_, _>>(),
        hound::SampleFormat::Int => {
            let scale = 1.0 / (1u64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 * scale))
                .collect::<Result<_, _>>()
        }
    }
    .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Audio {
        samples: mix(&interleaved, channels),
        sample_rate: spec.sample_rate as f32,
    })
}

fn load_flac(path: &Path) -> Result<Audio, String> {
    let mut reader =
        claxon::FlacReader::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let info = reader.streaminfo();
    let scale = 1.0 / (1u64 << (info.bits_per_sample - 1)) as f32;
    let interleaved: Vec<f32> = reader
        .samples()
        .map(|s| s.map(|v| v as f32 * scale))
        .collect::<Result<_, _>>()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Audio {
        samples: mix(&interleaved, info.channels as usize),
        sample_rate: info.sample_rate as f32,
    })
}

fn mix(interleaved: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return interleaved.to_vec();
    }
    interleaved
        .chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Writes mono 16-bit WAV (used for synthetic takes).
pub fn write_wav(path: &Path, audio: &Audio) -> Result<(), String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: audio.sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w =
        hound::WavWriter::create(path, spec).map_err(|e| format!("{}: {e}", path.display()))?;
    for &s in &audio.samples {
        w.write_sample((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
            .map_err(|e| e.to_string())?;
    }
    w.finalize().map_err(|e| e.to_string())
}
