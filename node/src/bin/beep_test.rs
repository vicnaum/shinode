//! IBM POST beep test - zero dependencies
//!
//! Run with: cargo run --manifest-path node/Cargo.toml --bin beep-test

use std::io::Write;

/// Generate IBM POST beep as WAV bytes (896.46 Hz square wave, ~230ms).
fn generate_post_beep_wav() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44100;
    const DURATION_MS: u32 = 230;
    const FREQ: f32 = 896.46;
    const AMPLITUDE: i16 = 1600; // ~0.05 volume
    const DUTY: f32 = 0.500376;

    let num_samples = (SAMPLE_RATE * DURATION_MS / 1000) as usize;
    let mut samples = Vec::with_capacity(num_samples * 2);

    let mut phase: f32 = 0.0;
    for _ in 0..num_samples {
        phase += FREQ / SAMPLE_RATE as f32;
        phase %= 1.0;
        let sample: i16 = if phase < DUTY { AMPLITUDE } else { -AMPLITUDE };
        samples.extend_from_slice(&sample.to_le_bytes());
    }

    let data_size = samples.len() as u32;
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + samples.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    wav.extend_from_slice(&samples);

    wav
}

fn main() {
    println!("Playing IBM POST beep...");

    let wav = generate_post_beep_wav();
    let temp_path = "/tmp/beep_test.wav";

    let mut file = std::fs::File::create(temp_path).expect("Failed to create temp file");
    file.write_all(&wav).expect("Failed to write WAV");

    std::process::Command::new("afplay")
        .arg(temp_path)
        .status()
        .expect("Failed to play sound");

    println!("Done!");
}
