mod analyzer;
mod db;
mod models;
mod navidrome;
mod scanner;

use db::Database;
use models::{MoodCategory, Playlist, Track};
use navidrome::{NavidromeAuth, SubsonicClient};
use scanner::LibraryScanner;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Mutex;
use tauri::State;

fn percent_decode(input: &str) -> String {
    let mut bytes = Vec::new();
    let src = input.as_bytes();
    let mut i = 0;
    while i < src.len() {
        if src[i] == b'%' && i + 2 < src.len() {
            if let Ok(byte) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                bytes.push(byte);
                i += 3;
                continue;
            }
        }
        bytes.push(src[i]);
        i += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn audio_content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().map(|e| e.to_lowercase()).as_deref() {
        Some("mp3") => "audio/mpeg",
        Some("flac") => "audio/flac",
        Some("ogg") => "audio/ogg",
        Some("opus") => "audio/opus",
        Some("wav") => "audio/wav",
        Some("aac") => "audio/aac",
        Some("m4a") => "audio/mp4",
        Some("wma") => "audio/x-ms-wma",
        Some("aiff") | Some("aif") => "audio/aiff",
        Some("ape") => "audio/x-ape",
        _ => "application/octet-stream",
    }
}

fn serve_audio(request: tauri::http::Request<Vec<u8>>) -> tauri::http::Response<Vec<u8>> {
    let raw_path = percent_decode(request.uri().path().trim_start_matches('/'));

    let mut file = match std::fs::File::open(&raw_path) {
        Ok(f) => f,
        Err(_) => {
            return tauri::http::Response::builder()
                .status(404)
                .body(Vec::new())
                .unwrap();
        }
    };
    let file_size = match file.metadata() {
        Ok(m) => m.len(),
        Err(_) => {
            return tauri::http::Response::builder()
                .status(500)
                .body(Vec::new())
                .unwrap();
        }
    };
    let content_type = audio_content_type(&raw_path);

    // Handle Range requests for streaming playback
    if let Some(range_header) = request.headers().get("range").and_then(|v| v.to_str().ok()) {
        if let Some(spec) = range_header.strip_prefix("bytes=") {
            let parts: Vec<&str> = spec.splitn(2, '-').collect();
            let start: u64 = parts[0].parse().unwrap_or(0);
            let end: u64 = if parts.len() > 1 && !parts[1].is_empty() {
                parts[1]
                    .parse::<u64>()
                    .unwrap_or(file_size - 1)
                    .min(file_size - 1)
            } else {
                file_size - 1
            };

            if start >= file_size {
                return tauri::http::Response::builder()
                    .status(416)
                    .header("Content-Range", format!("bytes */{file_size}"))
                    .body(Vec::new())
                    .unwrap();
            }

            let length = end - start + 1;
            if file.seek(SeekFrom::Start(start)).is_err() {
                return tauri::http::Response::builder()
                    .status(500)
                    .body(Vec::new())
                    .unwrap();
            }
            let mut buf = vec![0u8; length as usize];
            if file.read_exact(&mut buf).is_err() {
                return tauri::http::Response::builder()
                    .status(500)
                    .body(Vec::new())
                    .unwrap();
            }

            return tauri::http::Response::builder()
                .status(206)
                .header("Content-Type", content_type)
                .header("Content-Length", length.to_string())
                .header(
                    "Content-Range",
                    format!("bytes {start}-{end}/{file_size}"),
                )
                .header("Accept-Ranges", "bytes")
                .body(buf)
                .unwrap();
        }
    }

    // Full file response
    let mut data = Vec::with_capacity(file_size as usize);
    if file.read_to_end(&mut data).is_err() {
        return tauri::http::Response::builder()
            .status(500)
            .body(Vec::new())
            .unwrap();
    }

    tauri::http::Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .header("Content-Length", file_size.to_string())
        .header("Accept-Ranges", "bytes")
        .body(data)
        .unwrap()
}

struct AppState {
    db: Mutex<Database>,
}

struct NavidromeState {
    client: Mutex<Option<SubsonicClient>>,
}

#[tauri::command]
async fn scan_directory(path: String, state: State<'_, AppState>) -> Result<Vec<Track>, String> {
    let tracks = LibraryScanner::scan(&path).map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    for track in &tracks {
        db.insert_track(track).map_err(|e| e.to_string())?;
    }
    Ok(tracks)
}

#[tauri::command]
async fn get_all_tracks(state: State<'_, AppState>) -> Result<Vec<Track>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_tracks().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_artists(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_artists().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_albums(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_albums().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_genres(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_genres().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_tracks_by_artist(
    artist: String,
    state: State<'_, AppState>,
) -> Result<Vec<Track>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_tracks_by_artist(&artist).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_tracks_by_album(
    album: String,
    state: State<'_, AppState>,
) -> Result<Vec<Track>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_tracks_by_album(&album).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_tracks_by_genre(
    genre: String,
    state: State<'_, AppState>,
) -> Result<Vec<Track>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_tracks_by_genre(&genre).map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_tracks(query: String, state: State<'_, AppState>) -> Result<Vec<Track>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.search_tracks(&query).map_err(|e| e.to_string())
}

#[tauri::command]
async fn analyze_track(path: String, state: State<'_, AppState>) -> Result<Track, String> {
    let analysis = analyzer::analyze_file(&path).map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_track_analysis(&path, &analysis)
        .map_err(|e| e.to_string())?;
    db.get_track_by_path(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn analyze_all_tracks(state: State<'_, AppState>) -> Result<Vec<Track>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let tracks = db
        .get_unanalyzed_tracks()
        .map_err(|e| e.to_string())?;
    drop(db);

    for track in &tracks {
        if let Ok(analysis) = analyzer::analyze_file(&track.path) {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let _ = db.update_track_analysis(&track.path, &analysis);
        }
    }

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_tracks().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_tracks_by_mood(
    mood: MoodCategory,
    state: State<'_, AppState>,
) -> Result<Vec<Track>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_tracks_by_mood(&mood).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_mood_categories() -> Vec<MoodCategory> {
    MoodCategory::all()
}

#[tauri::command]
async fn create_playlist(
    name: String,
    state: State<'_, AppState>,
) -> Result<Playlist, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_playlist(&name).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_playlists(state: State<'_, AppState>) -> Result<Vec<Playlist>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_playlists().map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_to_playlist(
    playlist_id: String,
    track_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.add_to_playlist(&playlist_id, &track_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_from_playlist(
    playlist_id: String,
    track_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.remove_from_playlist(&playlist_id, &track_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_playlist_tracks(
    playlist_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Track>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_playlist_tracks(&playlist_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_playlist(playlist_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_playlist(&playlist_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_track_artwork(path: String) -> Result<Option<String>, String> {
    scanner::extract_artwork(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn increment_play_count(
    track_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.increment_play_count(&track_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_rating(
    track_id: String,
    rating: u8,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_rating(&track_id, rating)
        .map_err(|e| e.to_string())
}

// --- Navidrome / Subsonic commands ---

fn get_nd_client(state: &State<'_, NavidromeState>) -> Result<SubsonicClient, String> {
    let guard = state.client.lock().map_err(|e| e.to_string())?;
    guard
        .as_ref()
        .cloned()
        .ok_or_else(|| "Not connected to Navidrome".to_string())
}

#[tauri::command]
async fn nd_connect(
    url: String,
    username: String,
    password: String,
    state: State<'_, NavidromeState>,
) -> Result<NavidromeAuth, String> {
    let client = SubsonicClient::new(&url, &username, &password);
    client.ping().await?;
    let auth = client.auth();
    *state.client.lock().map_err(|e| e.to_string())? = Some(client);
    Ok(auth)
}

#[tauri::command]
async fn nd_disconnect(state: State<'_, NavidromeState>) -> Result<(), String> {
    *state.client.lock().map_err(|e| e.to_string())? = None;
    Ok(())
}

#[tauri::command]
async fn nd_get_all_tracks(state: State<'_, NavidromeState>) -> Result<Vec<Track>, String> {
    let client = get_nd_client(&state)?;
    client.get_all_songs().await
}

#[tauri::command]
async fn nd_get_artists(state: State<'_, NavidromeState>) -> Result<Vec<String>, String> {
    let client = get_nd_client(&state)?;
    client.get_artists().await
}

#[tauri::command]
async fn nd_get_albums(state: State<'_, NavidromeState>) -> Result<Vec<String>, String> {
    let client = get_nd_client(&state)?;
    client.get_albums().await
}

#[tauri::command]
async fn nd_get_genres(state: State<'_, NavidromeState>) -> Result<Vec<String>, String> {
    let client = get_nd_client(&state)?;
    client.get_genres().await
}

#[tauri::command]
async fn nd_search(query: String, state: State<'_, NavidromeState>) -> Result<Vec<Track>, String> {
    let client = get_nd_client(&state)?;
    client.search(&query).await
}

#[tauri::command]
async fn nd_get_playlists(state: State<'_, NavidromeState>) -> Result<Vec<Playlist>, String> {
    let client = get_nd_client(&state)?;
    client.get_playlists().await
}

#[tauri::command]
async fn nd_get_playlist_tracks(
    playlist_id: String,
    state: State<'_, NavidromeState>,
) -> Result<Vec<Track>, String> {
    let client = get_nd_client(&state)?;
    client.get_playlist_songs(&playlist_id).await
}

#[tauri::command]
async fn nd_create_playlist(
    name: String,
    state: State<'_, NavidromeState>,
) -> Result<Playlist, String> {
    let client = get_nd_client(&state)?;
    client.create_playlist(&name).await
}

#[tauri::command]
async fn nd_delete_playlist(
    playlist_id: String,
    state: State<'_, NavidromeState>,
) -> Result<(), String> {
    let client = get_nd_client(&state)?;
    client.delete_playlist(&playlist_id).await
}

#[tauri::command]
async fn nd_add_to_playlist(
    playlist_id: String,
    track_id: String,
    state: State<'_, NavidromeState>,
) -> Result<(), String> {
    let client = get_nd_client(&state)?;
    client.add_to_playlist(&playlist_id, &track_id).await
}

#[tauri::command]
async fn nd_remove_from_playlist(
    playlist_id: String,
    track_id: String,
    state: State<'_, NavidromeState>,
) -> Result<(), String> {
    let client = get_nd_client(&state)?;
    client.remove_from_playlist(&playlist_id, &track_id).await
}

#[tauri::command]
async fn nd_scrobble(track_id: String, state: State<'_, NavidromeState>) -> Result<(), String> {
    let client = get_nd_client(&state)?;
    client.scrobble(&track_id).await
}

#[tauri::command]
async fn nd_set_rating(
    track_id: String,
    rating: u8,
    state: State<'_, NavidromeState>,
) -> Result<(), String> {
    let client = get_nd_client(&state)?;
    client.set_rating(&track_id, rating).await
}

pub fn run() {
    let db = Database::new().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState { db: Mutex::new(db) })
        .manage(NavidromeState {
            client: Mutex::new(None),
        })
        .register_uri_scheme_protocol("heki-audio", |_app, request| serve_audio(request))
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            get_all_tracks,
            get_artists,
            get_albums,
            get_genres,
            get_tracks_by_artist,
            get_tracks_by_album,
            get_tracks_by_genre,
            search_tracks,
            analyze_track,
            analyze_all_tracks,
            get_tracks_by_mood,
            get_mood_categories,
            create_playlist,
            get_playlists,
            add_to_playlist,
            remove_from_playlist,
            get_playlist_tracks,
            delete_playlist,
            get_track_artwork,
            increment_play_count,
            update_rating,
            nd_connect,
            nd_disconnect,
            nd_get_all_tracks,
            nd_get_artists,
            nd_get_albums,
            nd_get_genres,
            nd_search,
            nd_get_playlists,
            nd_get_playlist_tracks,
            nd_create_playlist,
            nd_delete_playlist,
            nd_add_to_playlist,
            nd_remove_from_playlist,
            nd_scrobble,
            nd_set_rating,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
