use crate::models::{Playlist, Track};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone)]
pub struct SubsonicClient {
    base_url: String,
    username: String,
    token: String,
    salt: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize)]
pub struct NavidromeAuth {
    pub url: String,
    pub auth_params: String,
}

impl SubsonicClient {
    pub fn new(url: &str, username: &str, password: &str) -> Self {
        let salt = uuid::Uuid::new_v4().to_string().replace('-', "");
        let token = format!("{:x}", md5::compute(format!("{}{}", password, salt)));
        SubsonicClient {
            base_url: url.trim_end_matches('/').to_string(),
            username: username.to_string(),
            token,
            salt,
            client: reqwest::Client::new(),
        }
    }

    pub fn auth(&self) -> NavidromeAuth {
        NavidromeAuth {
            url: self.base_url.clone(),
            auth_params: format!(
                "u={}&t={}&s={}&v=1.16.1&c=heki",
                self.username, self.token, self.salt
            ),
        }
    }

    async fn get(&self, endpoint: &str, extra_params: &[(&str, &str)]) -> Result<Value, String> {
        let url = format!("{}/rest/{}", self.base_url, endpoint);
        let resp = self
            .client
            .get(&url)
            .query(&[
                ("u", self.username.as_str()),
                ("t", self.token.as_str()),
                ("s", self.salt.as_str()),
                ("v", "1.16.1"),
                ("c", "heki"),
                ("f", "json"),
            ])
            .query(extra_params)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let json: Value = resp
            .json()
            .await
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        let inner = json
            .get("subsonic-response")
            .ok_or_else(|| "Invalid Subsonic response".to_string())?;

        if inner["status"].as_str() != Some("ok") {
            let msg = inner["error"]["message"]
                .as_str()
                .unwrap_or("Unknown error");
            return Err(msg.to_string());
        }

        Ok(inner.clone())
    }

    pub async fn ping(&self) -> Result<(), String> {
        self.get("ping", &[]).await?;
        Ok(())
    }

    pub async fn get_all_songs(&self) -> Result<Vec<Track>, String> {
        let mut all_songs = Vec::new();
        let mut offset = 0u32;
        let count = 500u32;

        loop {
            let offset_str = offset.to_string();
            let count_str = count.to_string();
            let resp = self
                .get(
                    "search3",
                    &[
                        ("query", ""),
                        ("songCount", &count_str),
                        ("songOffset", &offset_str),
                        ("artistCount", "0"),
                        ("albumCount", "0"),
                    ],
                )
                .await?;

            let songs = resp
                .get("searchResult3")
                .and_then(|r| r.get("song"))
                .and_then(|s| s.as_array());

            match songs {
                Some(arr) if !arr.is_empty() => {
                    for song in arr {
                        all_songs.push(subsonic_song_to_track(song));
                    }
                    if arr.len() < count as usize {
                        break;
                    }
                    offset += count;
                }
                _ => break,
            }
        }

        Ok(all_songs)
    }

    pub async fn get_artists(&self) -> Result<Vec<String>, String> {
        let resp = self.get("getArtists", &[]).await?;
        let mut artists = Vec::new();
        if let Some(indices) = resp
            .get("artists")
            .and_then(|a| a.get("index"))
            .and_then(|i| i.as_array())
        {
            for index in indices {
                if let Some(artist_list) = index.get("artist").and_then(|a| a.as_array()) {
                    for artist in artist_list {
                        if let Some(name) = artist["name"].as_str() {
                            artists.push(name.to_string());
                        }
                    }
                }
            }
        }
        Ok(artists)
    }

    pub async fn get_albums(&self) -> Result<Vec<String>, String> {
        let mut albums = Vec::new();
        let mut offset = 0u32;
        let count = 500u32;
        loop {
            let offset_str = offset.to_string();
            let count_str = count.to_string();
            let resp = self
                .get(
                    "getAlbumList2",
                    &[
                        ("type", "alphabeticalByName"),
                        ("size", &count_str),
                        ("offset", &offset_str),
                    ],
                )
                .await?;

            let album_list = resp
                .get("albumList2")
                .and_then(|a| a.get("album"))
                .and_then(|a| a.as_array());

            match album_list {
                Some(arr) if !arr.is_empty() => {
                    for album in arr {
                        if let Some(name) = album["name"].as_str() {
                            albums.push(name.to_string());
                        }
                    }
                    if arr.len() < count as usize {
                        break;
                    }
                    offset += count;
                }
                _ => break,
            }
        }
        Ok(albums)
    }

    pub async fn get_genres(&self) -> Result<Vec<String>, String> {
        let resp = self.get("getGenres", &[]).await?;
        let mut genres = Vec::new();
        if let Some(genre_list) = resp
            .get("genres")
            .and_then(|g| g.get("genre"))
            .and_then(|g| g.as_array())
        {
            for genre in genre_list {
                if let Some(name) = genre["value"].as_str() {
                    if !name.is_empty() {
                        genres.push(name.to_string());
                    }
                }
            }
        }
        Ok(genres)
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Track>, String> {
        let resp = self
            .get(
                "search3",
                &[
                    ("query", query),
                    ("songCount", "100"),
                    ("artistCount", "0"),
                    ("albumCount", "0"),
                ],
            )
            .await?;

        let mut tracks = Vec::new();
        if let Some(songs) = resp
            .get("searchResult3")
            .and_then(|r| r.get("song"))
            .and_then(|s| s.as_array())
        {
            for song in songs {
                tracks.push(subsonic_song_to_track(song));
            }
        }
        Ok(tracks)
    }

    pub async fn get_playlists(&self) -> Result<Vec<Playlist>, String> {
        let resp = self.get("getPlaylists", &[]).await?;
        let mut playlists = Vec::new();
        if let Some(pl_list) = resp
            .get("playlists")
            .and_then(|p| p.get("playlist"))
            .and_then(|p| p.as_array())
        {
            for pl in pl_list {
                playlists.push(subsonic_playlist_to_playlist(pl));
            }
        }
        Ok(playlists)
    }

    pub async fn get_playlist_songs(&self, id: &str) -> Result<Vec<Track>, String> {
        let resp = self.get("getPlaylist", &[("id", id)]).await?;
        let mut tracks = Vec::new();
        if let Some(entries) = resp
            .get("playlist")
            .and_then(|p| p.get("entry"))
            .and_then(|e| e.as_array())
        {
            for entry in entries {
                tracks.push(subsonic_song_to_track(entry));
            }
        }
        Ok(tracks)
    }

    pub async fn create_playlist(&self, name: &str) -> Result<Playlist, String> {
        let resp = self.get("createPlaylist", &[("name", name)]).await?;
        if let Some(pl) = resp.get("playlist") {
            Ok(subsonic_playlist_to_playlist(pl))
        } else {
            // Some servers don't return the playlist object; fetch all and find by name
            let playlists = self.get_playlists().await?;
            playlists
                .into_iter()
                .find(|p| p.name == name)
                .ok_or_else(|| "Playlist created but not found".to_string())
        }
    }

    pub async fn delete_playlist(&self, id: &str) -> Result<(), String> {
        self.get("deletePlaylist", &[("id", id)]).await?;
        Ok(())
    }

    pub async fn add_to_playlist(&self, playlist_id: &str, song_id: &str) -> Result<(), String> {
        self.get(
            "updatePlaylist",
            &[("playlistId", playlist_id), ("songIdToAdd", song_id)],
        )
        .await?;
        Ok(())
    }

    pub async fn remove_from_playlist(
        &self,
        playlist_id: &str,
        track_id: &str,
    ) -> Result<(), String> {
        // Subsonic API needs the song index (position), not the song ID
        let resp = self.get("getPlaylist", &[("id", playlist_id)]).await?;
        let entries = resp
            .get("playlist")
            .and_then(|p| p.get("entry"))
            .and_then(|e| e.as_array())
            .ok_or_else(|| "No entries in playlist".to_string())?;

        let index = entries
            .iter()
            .position(|e| e["id"].as_str() == Some(track_id))
            .ok_or_else(|| "Song not found in playlist".to_string())?;

        let index_str = index.to_string();
        self.get(
            "updatePlaylist",
            &[
                ("playlistId", playlist_id),
                ("songIndexToRemove", &index_str),
            ],
        )
        .await?;
        Ok(())
    }

    pub async fn scrobble(&self, song_id: &str) -> Result<(), String> {
        self.get("scrobble", &[("id", song_id)]).await?;
        Ok(())
    }

    pub async fn set_rating(&self, song_id: &str, rating: u8) -> Result<(), String> {
        let rating_str = rating.to_string();
        self.get("setRating", &[("id", song_id), ("rating", &rating_str)])
            .await?;
        Ok(())
    }

    pub fn stream_url(&self, song_id: &str) -> String {
        format!(
            "{}/rest/stream?id={}&{}",
            self.base_url,
            song_id,
            self.auth().auth_params
        )
    }

    pub fn cover_art_url(&self, id: &str) -> String {
        format!(
            "{}/rest/getCoverArt?id={}&size=300&{}",
            self.base_url,
            id,
            self.auth().auth_params
        )
    }
}

fn subsonic_song_to_track(song: &Value) -> Track {
    let id = song["id"].as_str().unwrap_or("").to_string();
    Track {
        id: id.clone(),
        path: format!("navidrome://{}", id),
        title: song["title"].as_str().unwrap_or("Unknown").to_string(),
        artist: song["artist"].as_str().unwrap_or("Unknown").to_string(),
        album: song["album"].as_str().unwrap_or("Unknown").to_string(),
        album_artist: song["albumArtist"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        genre: song["genre"].as_str().unwrap_or("").to_string(),
        track_number: song["track"].as_u64().map(|n| n as u32),
        disc_number: song["discNumber"].as_u64().map(|n| n as u32),
        year: song["year"].as_u64().map(|n| n as u32),
        duration_secs: song["duration"].as_f64().unwrap_or(0.0),
        file_format: song["suffix"]
            .as_str()
            .unwrap_or("")
            .to_uppercase(),
        bitrate: song["bitRate"].as_u64().map(|n| n as u32),
        sample_rate: song["samplingRate"].as_u64().map(|n| n as u32),
        bpm: None,
        energy: None,
        valence: None,
        mood: None,
        play_count: song["playCount"].as_u64().unwrap_or(0) as u32,
        rating: song["userRating"].as_u64().unwrap_or(0) as u8,
        date_added: song["created"].as_str().unwrap_or("").to_string(),
        has_artwork: song["coverArt"].as_str().is_some(),
    }
}

fn subsonic_playlist_to_playlist(pl: &Value) -> Playlist {
    Playlist {
        id: pl["id"].as_str().unwrap_or("").to_string(),
        name: pl["name"].as_str().unwrap_or("").to_string(),
        track_count: pl["songCount"].as_u64().unwrap_or(0) as u32,
        is_smart: false,
        mood_filter: None,
        created_at: pl["created"].as_str().unwrap_or("").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsonic_song_to_track_basic() {
        let song: Value = serde_json::json!({
            "id": "abc123",
            "title": "Test Song",
            "artist": "Test Artist",
            "album": "Test Album",
            "genre": "Rock",
            "track": 3,
            "year": 2024,
            "duration": 240,
            "suffix": "flac",
            "bitRate": 1411,
            "coverArt": "al-xyz",
            "created": "2024-01-15T10:30:00Z"
        });
        let track = subsonic_song_to_track(&song);
        assert_eq!(track.id, "abc123");
        assert_eq!(track.path, "navidrome://abc123");
        assert_eq!(track.title, "Test Song");
        assert_eq!(track.artist, "Test Artist");
        assert_eq!(track.album, "Test Album");
        assert_eq!(track.genre, "Rock");
        assert_eq!(track.track_number, Some(3));
        assert_eq!(track.year, Some(2024));
        assert_eq!(track.duration_secs, 240.0);
        assert_eq!(track.file_format, "FLAC");
        assert_eq!(track.bitrate, Some(1411));
        assert!(track.has_artwork);
        assert!(track.mood.is_none());
    }

    #[test]
    fn subsonic_song_to_track_missing_fields() {
        let song: Value = serde_json::json!({
            "id": "def456",
            "title": "Minimal"
        });
        let track = subsonic_song_to_track(&song);
        assert_eq!(track.id, "def456");
        assert_eq!(track.title, "Minimal");
        assert_eq!(track.artist, "Unknown");
        assert_eq!(track.album, "Unknown");
        assert_eq!(track.duration_secs, 0.0);
        assert!(!track.has_artwork);
    }

    #[test]
    fn subsonic_playlist_to_playlist_basic() {
        let pl: Value = serde_json::json!({
            "id": "pl001",
            "name": "My Playlist",
            "songCount": 42,
            "created": "2024-06-01T00:00:00Z"
        });
        let playlist = subsonic_playlist_to_playlist(&pl);
        assert_eq!(playlist.id, "pl001");
        assert_eq!(playlist.name, "My Playlist");
        assert_eq!(playlist.track_count, 42);
        assert!(!playlist.is_smart);
        assert!(playlist.mood_filter.is_none());
    }

    #[test]
    fn navidrome_auth_params_format() {
        let client = SubsonicClient::new("https://music.example.com", "testuser", "password123");
        let auth = client.auth();
        assert_eq!(auth.url, "https://music.example.com");
        assert!(auth.auth_params.starts_with("u=testuser&t="));
        assert!(auth.auth_params.contains("&s="));
        assert!(auth.auth_params.ends_with("&v=1.16.1&c=heki"));
    }

    #[test]
    fn stream_url_format() {
        let client = SubsonicClient::new("https://nd.example.com", "user", "pass");
        let url = client.stream_url("song123");
        assert!(url.starts_with("https://nd.example.com/rest/stream?id=song123&u=user&t="));
    }

    #[test]
    fn cover_art_url_format() {
        let client = SubsonicClient::new("https://nd.example.com", "user", "pass");
        let url = client.cover_art_url("al-abc");
        assert!(url.starts_with("https://nd.example.com/rest/getCoverArt?id=al-abc&size=300&u=user&t="));
    }
}
