use crate::models::{AudioAnalysis, MoodCategory};
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub fn analyze_file(path: &str) -> Result<AudioAnalysis, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(path).extension() {
        hint.with_extension(&ext.to_string_lossy());
    }

    let probed = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;

    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or("no default track")?;
    let track_id = track.id;

    let sample_rate = track
        .codec_params
        .sample_rate
        .unwrap_or(44100) as f64;

    let mut decoder = symphonia::default::get_codecs().make(
        &track.codec_params,
        &DecoderOptions::default(),
    )?;

    let mut all_samples: Vec<f32> = Vec::new();
    let max_samples = (sample_rate * 60.0) as usize; // analyze up to 60 seconds

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(_) => continue,
        };

        let spec = *decoded.spec();
        let num_frames = decoded.frames();
        let mut sample_buf = SampleBuffer::<f32>::new(num_frames as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);
        let samples = sample_buf.samples();

        // Convert to mono by averaging channels
        let channels = spec.channels.count();
        for chunk in samples.chunks(channels) {
            let mono: f32 = chunk.iter().sum::<f32>() / channels as f32;
            all_samples.push(mono);
        }

        if all_samples.len() >= max_samples {
            break;
        }
    }

    if all_samples.is_empty() {
        return Err("No audio samples decoded".into());
    }

    let bpm = detect_tempo(&all_samples, sample_rate);
    let energy = calculate_energy(&all_samples);
    let valence = calculate_valence(&all_samples, sample_rate);
    let mood = MoodCategory::from_features(bpm, energy, valence);

    Ok(AudioAnalysis {
        bpm,
        energy,
        valence,
        mood,
    })
}

fn detect_tempo(samples: &[f32], sample_rate: f64) -> f64 {
    // Simple tempo detection using onset detection and autocorrelation
    let hop_size = (sample_rate * 0.01) as usize; // 10ms hops
    let frame_size = (sample_rate * 0.025) as usize; // 25ms frames

    // Calculate spectral flux (onset detection function)
    let mut onset_env: Vec<f64> = Vec::new();
    let mut prev_energy = 0.0f64;

    for start in (0..samples.len().saturating_sub(frame_size)).step_by(hop_size) {
        let frame = &samples[start..start + frame_size];
        let frame_energy: f64 = frame.iter().map(|s| (*s as f64).powi(2)).sum::<f64>()
            / frame_size as f64;
        let flux = (frame_energy - prev_energy).max(0.0);
        onset_env.push(flux);
        prev_energy = frame_energy;
    }

    if onset_env.len() < 100 {
        return 120.0; // default BPM
    }

    // Autocorrelation on onset envelope
    let min_lag = (60.0 / 200.0 * (sample_rate / hop_size as f64)) as usize; // 200 BPM
    let max_lag = (60.0 / 50.0 * (sample_rate / hop_size as f64)) as usize; // 50 BPM
    let max_lag = max_lag.min(onset_env.len() / 2);

    let mut best_lag = min_lag;
    let mut best_corr = 0.0f64;

    let mean: f64 = onset_env.iter().sum::<f64>() / onset_env.len() as f64;

    for lag in min_lag..max_lag {
        let mut corr = 0.0f64;
        let n = onset_env.len() - lag;
        for i in 0..n {
            corr += (onset_env[i] - mean) * (onset_env[i + lag] - mean);
        }
        corr /= n as f64;

        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }

    let bpm = 60.0 / (best_lag as f64 * hop_size as f64 / sample_rate);

    // Clamp to reasonable range
    if bpm < 50.0 {
        bpm * 2.0
    } else if bpm > 200.0 {
        bpm / 2.0
    } else {
        bpm
    }
}

fn calculate_energy(samples: &[f32]) -> f64 {
    // RMS energy normalized to 0-1 range
    let rms: f64 = (samples.iter().map(|s| (*s as f64).powi(2)).sum::<f64>()
        / samples.len() as f64)
        .sqrt();

    // Normalize: typical RMS for music is 0.05-0.3
    let normalized = ((rms - 0.02) / 0.28).clamp(0.0, 1.0);

    // Also consider dynamic range
    let peak: f64 = samples
        .iter()
        .map(|s| s.abs() as f64)
        .fold(0.0, f64::max);
    let crest_factor = if rms > 0.001 { peak / rms } else { 10.0 };

    // Lower crest factor = more compressed = more "energetic" feel
    let compression_factor = (1.0 - (crest_factor - 1.0) / 20.0).clamp(0.0, 1.0);

    (normalized * 0.7 + compression_factor * 0.3).clamp(0.0, 1.0)
}

fn calculate_valence(samples: &[f32], sample_rate: f64) -> f64 {
    // Estimate valence using spectral features
    // Higher spectral centroid and more high-frequency content = higher valence (brighter, happier)

    let frame_size = 2048usize;
    let hop_size = 1024usize;
    let mut spectral_centroids: Vec<f64> = Vec::new();
    let mut spectral_brightness: Vec<f64> = Vec::new();

    for start in (0..samples.len().saturating_sub(frame_size)).step_by(hop_size) {
        let frame = &samples[start..start + frame_size];

        // Simple DFT magnitude spectrum (just compute a few frequency bands)
        let n_bands = 16;
        let mut band_energies = vec![0.0f64; n_bands];

        for (band_idx, energy) in band_energies.iter_mut().enumerate() {
            let freq = (band_idx as f64 + 0.5) * sample_rate / (2.0 * n_bands as f64);
            let mut real = 0.0f64;
            let mut imag = 0.0f64;
            for (i, &sample) in frame.iter().enumerate() {
                let angle = 2.0 * std::f64::consts::PI * freq * i as f64 / sample_rate;
                real += sample as f64 * angle.cos();
                imag += sample as f64 * angle.sin();
            }
            *energy = (real.powi(2) + imag.powi(2)).sqrt();
        }

        // Spectral centroid
        let total_energy: f64 = band_energies.iter().sum();
        if total_energy > 0.001 {
            let centroid: f64 = band_energies
                .iter()
                .enumerate()
                .map(|(i, e)| (i as f64 + 1.0) * e)
                .sum::<f64>()
                / total_energy;
            spectral_centroids.push(centroid / n_bands as f64);

            // Brightness: ratio of high frequency energy to total
            let high_energy: f64 = band_energies[n_bands / 2..].iter().sum();
            spectral_brightness.push(high_energy / total_energy);
        }
    }

    if spectral_centroids.is_empty() {
        return 0.5;
    }

    let avg_centroid: f64 =
        spectral_centroids.iter().sum::<f64>() / spectral_centroids.len() as f64;
    let avg_brightness: f64 =
        spectral_brightness.iter().sum::<f64>() / spectral_brightness.len() as f64;

    // Combine features for valence estimate
    let valence = (avg_centroid * 0.6 + avg_brightness * 0.4).clamp(0.0, 1.0);
    valence
}
