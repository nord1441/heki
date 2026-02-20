mod analyzer;
mod db;
mod models;
mod scanner;

use db::Database;
use models::{MoodCategory, Playlist, Track};
use scanner::LibraryScanner;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    db: Mutex<Database>,
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

pub fn run() {
    let db = Database::new().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState { db: Mutex::new(db) })
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
