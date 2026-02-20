use crate::models::Track;
use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::Path;
use uuid::Uuid;
use walkdir::WalkDir;

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "ogg", "opus", "wav", "aac", "m4a", "wma", "aiff", "ape",
];

pub struct LibraryScanner;

impl LibraryScanner {
    pub fn scan(dir: &str) -> Result<Vec<Track>, Box<dyn std::error::Error>> {
        let mut tracks = Vec::new();

        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str()) {
                        match Self::read_track(path) {
                            Ok(track) => tracks.push(track),
                            Err(e) => {
                                log::warn!("Failed to read {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        Ok(tracks)
    }

    fn read_track(path: &Path) -> Result<Track, Box<dyn std::error::Error>> {
        let tagged_file = Probe::open(path)?.read()?;

        let properties = tagged_file.properties();
        let duration = properties.duration();
        let bitrate = properties.audio_bitrate();
        let sample_rate = properties.sample_rate();

        let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

        let (title, artist, album, album_artist, genre, track_number, disc_number, year) =
            if let Some(tag) = tag {
                (
                    tag.title().map(|s| s.to_string()).unwrap_or_default(),
                    tag.artist().map(|s| s.to_string()).unwrap_or_default(),
                    tag.album().map(|s| s.to_string()).unwrap_or_default(),
                    tag.get_string(&ItemKey::AlbumArtist)
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    tag.genre().map(|s| s.to_string()).unwrap_or_default(),
                    tag.track().map(|n| n as u32),
                    tag.disk().map(|n| n as u32),
                    tag.year().map(|n| n as u32),
                )
            } else {
                let filename = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                (
                    filename,
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    None,
                    None,
                    None,
                )
            };

        let has_artwork = tag
            .map(|t| {
                t.pictures().len() > 0
            })
            .unwrap_or(false);

        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_uppercase())
            .unwrap_or_default();

        Ok(Track {
            id: Uuid::new_v4().to_string(),
            path: path.to_string_lossy().to_string(),
            title: if title.is_empty() {
                path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                title
            },
            artist: if artist.is_empty() {
                "Unknown Artist".to_string()
            } else {
                artist
            },
            album: if album.is_empty() {
                "Unknown Album".to_string()
            } else {
                album
            },
            album_artist,
            genre,
            track_number,
            disc_number,
            year,
            duration_secs: duration.as_secs_f64(),
            file_format: ext,
            bitrate,
            sample_rate,
            bpm: None,
            energy: None,
            valence: None,
            mood: None,
            play_count: 0,
            rating: 0,
            date_added: chrono::Utc::now().to_rfc3339(),
            has_artwork,
        })
    }
}

pub fn extract_artwork(path: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    use base64::Engine;

    let tagged_file = Probe::open(path)?.read()?;
    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

    if let Some(tag) = tag {
        if let Some(picture) = tag.pictures().first() {
            let mime = match picture.pic_type() {
                _ => {
                    let mime_str = picture.mime_type().unwrap_or(&lofty::picture::MimeType::Png);
                    match mime_str {
                        lofty::picture::MimeType::Png => "image/png",
                        lofty::picture::MimeType::Jpeg => "image/jpeg",
                        lofty::picture::MimeType::Bmp => "image/bmp",
                        lofty::picture::MimeType::Gif => "image/gif",
                        lofty::picture::MimeType::Tiff => "image/tiff",
                        _ => "image/jpeg",
                    }
                }
            };
            let b64 = base64::engine::general_purpose::STANDARD.encode(picture.data());
            return Ok(Some(format!("data:{};base64,{}", mime, b64)));
        }
    }

    Ok(None)
}
