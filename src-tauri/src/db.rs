use crate::models::{AudioAnalysis, MoodCategory, Playlist, Track};
use rusqlite::{params, Connection};
use uuid::Uuid;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let app_dir = dirs_next().unwrap_or_else(|| std::path::PathBuf::from("."));
        std::fs::create_dir_all(&app_dir)?;
        let db_path = app_dir.join("heki.db");
        let conn = Connection::open(db_path)?;
        let db = Database { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS tracks (
                id TEXT PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                artist TEXT NOT NULL DEFAULT '',
                album TEXT NOT NULL DEFAULT '',
                album_artist TEXT NOT NULL DEFAULT '',
                genre TEXT NOT NULL DEFAULT '',
                track_number INTEGER,
                disc_number INTEGER,
                year INTEGER,
                duration_secs REAL NOT NULL DEFAULT 0,
                file_format TEXT NOT NULL DEFAULT '',
                bitrate INTEGER,
                sample_rate INTEGER,
                bpm REAL,
                energy REAL,
                valence REAL,
                mood TEXT,
                play_count INTEGER NOT NULL DEFAULT 0,
                rating INTEGER NOT NULL DEFAULT 0,
                date_added TEXT NOT NULL DEFAULT '',
                has_artwork INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                is_smart INTEGER NOT NULL DEFAULT 0,
                mood_filter TEXT,
                created_at TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id TEXT NOT NULL,
                track_id TEXT NOT NULL,
                position INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (playlist_id, track_id),
                FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
                FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist);
            CREATE INDEX IF NOT EXISTS idx_tracks_album ON tracks(album);
            CREATE INDEX IF NOT EXISTS idx_tracks_genre ON tracks(genre);
            CREATE INDEX IF NOT EXISTS idx_tracks_mood ON tracks(mood);
            CREATE INDEX IF NOT EXISTS idx_tracks_path ON tracks(path);
            ",
        )
    }

    pub fn insert_track(&self, track: &Track) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tracks
             (id, path, title, artist, album, album_artist, genre, track_number, disc_number,
              year, duration_secs, file_format, bitrate, sample_rate, bpm, energy, valence,
              mood, play_count, rating, date_added, has_artwork)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                     ?17, ?18, ?19, ?20, ?21, ?22)",
            params![
                track.id,
                track.path,
                track.title,
                track.artist,
                track.album,
                track.album_artist,
                track.genre,
                track.track_number,
                track.disc_number,
                track.year,
                track.duration_secs,
                track.file_format,
                track.bitrate,
                track.sample_rate,
                track.bpm,
                track.energy,
                track.valence,
                track.mood.as_ref().map(|m| m.as_str()),
                track.play_count,
                track.rating,
                track.date_added,
                track.has_artwork as i32,
            ],
        )?;
        Ok(())
    }

    pub fn get_all_tracks(&self) -> Result<Vec<Track>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM tracks ORDER BY artist, album, track_number")?;
        let tracks = stmt
            .query_map([], |row| Self::row_to_track(row))?
            .filter_map(|t| t.ok())
            .collect();
        Ok(tracks)
    }

    pub fn get_track_by_path(&self, path: &str) -> Result<Track, rusqlite::Error> {
        self.conn
            .query_row("SELECT * FROM tracks WHERE path = ?1", params![path], |row| {
                Self::row_to_track(row)
            })
    }

    pub fn get_artists(&self) -> Result<Vec<String>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT artist FROM tracks WHERE artist != '' ORDER BY artist")?;
        let artists = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|a| a.ok())
            .collect();
        Ok(artists)
    }

    pub fn get_albums(&self) -> Result<Vec<String>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT album FROM tracks WHERE album != '' ORDER BY album")?;
        let albums = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|a| a.ok())
            .collect();
        Ok(albums)
    }

    pub fn get_genres(&self) -> Result<Vec<String>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT genre FROM tracks WHERE genre != '' ORDER BY genre")?;
        let genres = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|g| g.ok())
            .collect();
        Ok(genres)
    }

    pub fn get_tracks_by_artist(&self, artist: &str) -> Result<Vec<Track>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM tracks WHERE artist = ?1 ORDER BY album, track_number",
        )?;
        let tracks = stmt
            .query_map(params![artist], |row| Self::row_to_track(row))?
            .filter_map(|t| t.ok())
            .collect();
        Ok(tracks)
    }

    pub fn get_tracks_by_album(&self, album: &str) -> Result<Vec<Track>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM tracks WHERE album = ?1 ORDER BY track_number")?;
        let tracks = stmt
            .query_map(params![album], |row| Self::row_to_track(row))?
            .filter_map(|t| t.ok())
            .collect();
        Ok(tracks)
    }

    pub fn get_tracks_by_genre(&self, genre: &str) -> Result<Vec<Track>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM tracks WHERE genre = ?1 ORDER BY artist, album, track_number",
        )?;
        let tracks = stmt
            .query_map(params![genre], |row| Self::row_to_track(row))?
            .filter_map(|t| t.ok())
            .collect();
        Ok(tracks)
    }

    pub fn search_tracks(&self, query: &str) -> Result<Vec<Track>, rusqlite::Error> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT * FROM tracks WHERE title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1
             ORDER BY artist, album, track_number",
        )?;
        let tracks = stmt
            .query_map(params![pattern], |row| Self::row_to_track(row))?
            .filter_map(|t| t.ok())
            .collect();
        Ok(tracks)
    }

    pub fn get_unanalyzed_tracks(&self) -> Result<Vec<Track>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM tracks WHERE mood IS NULL")?;
        let tracks = stmt
            .query_map([], |row| Self::row_to_track(row))?
            .filter_map(|t| t.ok())
            .collect();
        Ok(tracks)
    }

    pub fn update_track_analysis(
        &self,
        path: &str,
        analysis: &AudioAnalysis,
    ) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE tracks SET bpm = ?1, energy = ?2, valence = ?3, mood = ?4 WHERE path = ?5",
            params![
                analysis.bpm,
                analysis.energy,
                analysis.valence,
                analysis.mood.as_str(),
                path,
            ],
        )?;
        Ok(())
    }

    pub fn get_tracks_by_mood(&self, mood: &MoodCategory) -> Result<Vec<Track>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM tracks WHERE mood = ?1 ORDER BY artist, album, track_number",
        )?;
        let tracks = stmt
            .query_map(params![mood.as_str()], |row| Self::row_to_track(row))?
            .filter_map(|t| t.ok())
            .collect();
        Ok(tracks)
    }

    pub fn create_playlist(&self, name: &str) -> Result<Playlist, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO playlists (id, name, is_smart, created_at) VALUES (?1, ?2, 0, ?3)",
            params![id, name, created_at],
        )?;
        Ok(Playlist {
            id,
            name: name.to_string(),
            track_count: 0,
            is_smart: false,
            mood_filter: None,
            created_at,
        })
    }

    pub fn get_playlists(&self) -> Result<Vec<Playlist>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.name, p.is_smart, p.mood_filter, p.created_at,
                    COUNT(pt.track_id) as track_count
             FROM playlists p
             LEFT JOIN playlist_tracks pt ON p.id = pt.playlist_id
             GROUP BY p.id
             ORDER BY p.name",
        )?;
        let playlists = stmt
            .query_map([], |row| {
                let mood_str: Option<String> = row.get(3)?;
                Ok(Playlist {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    is_smart: row.get::<_, i32>(2)? != 0,
                    mood_filter: mood_str.and_then(|s| MoodCategory::from_str(&s)),
                    track_count: row.get(5)?,
                    created_at: row.get(4)?,
                })
            })?
            .filter_map(|p| p.ok())
            .collect();
        Ok(playlists)
    }

    pub fn add_to_playlist(
        &self,
        playlist_id: &str,
        track_id: &str,
    ) -> Result<(), rusqlite::Error> {
        let max_pos: i32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(position), -1) FROM playlist_tracks WHERE playlist_id = ?1",
                params![playlist_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        self.conn.execute(
            "INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position)
             VALUES (?1, ?2, ?3)",
            params![playlist_id, track_id, max_pos + 1],
        )?;
        Ok(())
    }

    pub fn remove_from_playlist(
        &self,
        playlist_id: &str,
        track_id: &str,
    ) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
            params![playlist_id, track_id],
        )?;
        Ok(())
    }

    pub fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<Track>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT t.* FROM tracks t
             JOIN playlist_tracks pt ON t.id = pt.track_id
             WHERE pt.playlist_id = ?1
             ORDER BY pt.position",
        )?;
        let tracks = stmt
            .query_map(params![playlist_id], |row| Self::row_to_track(row))?
            .filter_map(|t| t.ok())
            .collect();
        Ok(tracks)
    }

    pub fn delete_playlist(&self, playlist_id: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "DELETE FROM playlist_tracks WHERE playlist_id = ?1",
            params![playlist_id],
        )?;
        self.conn.execute(
            "DELETE FROM playlists WHERE id = ?1",
            params![playlist_id],
        )?;
        Ok(())
    }

    pub fn increment_play_count(&self, track_id: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE tracks SET play_count = play_count + 1 WHERE id = ?1",
            params![track_id],
        )?;
        Ok(())
    }

    pub fn update_rating(&self, track_id: &str, rating: u8) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE tracks SET rating = ?1 WHERE id = ?2",
            params![rating, track_id],
        )?;
        Ok(())
    }

    fn row_to_track(row: &rusqlite::Row) -> Result<Track, rusqlite::Error> {
        let mood_str: Option<String> = row.get(17)?;
        Ok(Track {
            id: row.get(0)?,
            path: row.get(1)?,
            title: row.get(2)?,
            artist: row.get(3)?,
            album: row.get(4)?,
            album_artist: row.get(5)?,
            genre: row.get(6)?,
            track_number: row.get(7)?,
            disc_number: row.get(8)?,
            year: row.get(9)?,
            duration_secs: row.get(10)?,
            file_format: row.get(11)?,
            bitrate: row.get(12)?,
            sample_rate: row.get(13)?,
            bpm: row.get(14)?,
            energy: row.get(15)?,
            valence: row.get(16)?,
            mood: mood_str.and_then(|s| MoodCategory::from_str(&s)),
            play_count: row.get(18)?,
            rating: row.get(19)?,
            date_added: row.get(20)?,
            has_artwork: row.get::<_, i32>(21)? != 0,
        })
    }
}

/// テスト用: インメモリ DB を作成する
#[cfg(test)]
impl Database {
    pub fn new_in_memory() -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open_in_memory()?;
        let db = Database { conn };
        db.init_tables()?;
        Ok(db)
    }
}

fn dirs_next() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| std::path::PathBuf::from(h).join(".local/share"))
            })
            .map(|p| p.join("heki"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .ok()
            .map(|h| {
                std::path::PathBuf::from(h)
                    .join("Library/Application Support")
                    .join("heki")
            })
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .ok()
            .map(|h| std::path::PathBuf::from(h).join("heki"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Some(std::path::PathBuf::from("./heki_data"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AudioAnalysis, MoodCategory};

    fn sample_track(id: &str, path: &str) -> Track {
        Track {
            id: id.to_string(),
            path: path.to_string(),
            title: format!("Title {}", id),
            artist: "Test Artist".to_string(),
            album: "Test Album".to_string(),
            album_artist: String::new(),
            genre: "Rock".to_string(),
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2024),
            duration_secs: 200.0,
            file_format: "FLAC".to_string(),
            bitrate: Some(1411),
            sample_rate: Some(44100),
            bpm: None,
            energy: None,
            valence: None,
            mood: None,
            play_count: 0,
            rating: 0,
            date_added: "2024-01-01T00:00:00Z".to_string(),
            has_artwork: false,
        }
    }

    #[test]
    // インメモリ DB の初期化とテーブル作成が成功すること
    fn create_database_in_memory() {
        let db = Database::new_in_memory().unwrap();
        let tracks = db.get_all_tracks().unwrap();
        assert!(tracks.is_empty());
    }

    #[test]
    // トラックの挿入と取得が正しく動作すること
    fn insert_and_get_track() {
        let db = Database::new_in_memory().unwrap();
        let track = sample_track("t1", "/music/song.flac");
        db.insert_track(&track).unwrap();
        let tracks = db.get_all_tracks().unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Title t1");
    }

    #[test]
    // 同一パスのトラックを再挿入すると上書き (INSERT OR REPLACE) されること
    fn insert_duplicate_path_replaces() {
        let db = Database::new_in_memory().unwrap();
        let t1 = sample_track("t1", "/music/song.flac");
        db.insert_track(&t1).unwrap();
        let t2 = Track {
            id: "t2".to_string(),
            title: "Updated Title".to_string(),
            ..sample_track("t2", "/music/song.flac")
        };
        db.insert_track(&t2).unwrap();
        let tracks = db.get_all_tracks().unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Updated Title");
    }

    #[test]
    // get_track_by_path でパス指定取得できること
    fn get_track_by_path() {
        let db = Database::new_in_memory().unwrap();
        db.insert_track(&sample_track("t1", "/music/a.flac")).unwrap();
        db.insert_track(&sample_track("t2", "/music/b.flac")).unwrap();
        let track = db.get_track_by_path("/music/b.flac").unwrap();
        assert_eq!(track.id, "t2");
    }

    #[test]
    // アーティスト一覧が重複なく取得できること
    fn get_artists_distinct() {
        let db = Database::new_in_memory().unwrap();
        let mut t1 = sample_track("t1", "/a.flac");
        t1.artist = "Alpha".to_string();
        let mut t2 = sample_track("t2", "/b.flac");
        t2.artist = "Alpha".to_string();
        let mut t3 = sample_track("t3", "/c.flac");
        t3.artist = "Beta".to_string();
        db.insert_track(&t1).unwrap();
        db.insert_track(&t2).unwrap();
        db.insert_track(&t3).unwrap();
        let artists = db.get_artists().unwrap();
        assert_eq!(artists.len(), 2);
    }

    #[test]
    // アーティスト / アルバム / ジャンル別のフィルター取得が正しいこと
    fn get_tracks_by_artist_album_genre() {
        let db = Database::new_in_memory().unwrap();
        let mut t1 = sample_track("t1", "/a.flac");
        t1.artist = "A".to_string();
        t1.album = "X".to_string();
        t1.genre = "Pop".to_string();
        let mut t2 = sample_track("t2", "/b.flac");
        t2.artist = "B".to_string();
        t2.album = "Y".to_string();
        t2.genre = "Rock".to_string();
        db.insert_track(&t1).unwrap();
        db.insert_track(&t2).unwrap();

        assert_eq!(db.get_tracks_by_artist("A").unwrap().len(), 1);
        assert_eq!(db.get_tracks_by_album("Y").unwrap().len(), 1);
        assert_eq!(db.get_tracks_by_genre("Pop").unwrap().len(), 1);
    }

    #[test]
    // 検索がタイトル・アーティスト・アルバムにまたがる LIKE 検索であること
    fn search_tracks_matches_title_artist_album() {
        let db = Database::new_in_memory().unwrap();
        let mut t1 = sample_track("t1", "/a.flac");
        t1.title = "Moonlight Sonata".to_string();
        t1.artist = "Beethoven".to_string();
        let mut t2 = sample_track("t2", "/b.flac");
        t2.title = "Clair de Lune".to_string();
        t2.album = "Moonlight Collection".to_string();
        db.insert_track(&t1).unwrap();
        db.insert_track(&t2).unwrap();

        let results = db.search_tracks("moon").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    // 解析結果の更新でムード情報が反映されること
    fn update_track_analysis() {
        let db = Database::new_in_memory().unwrap();
        db.insert_track(&sample_track("t1", "/a.flac")).unwrap();
        let analysis = AudioAnalysis {
            bpm: 128.0,
            energy: 0.8,
            valence: 0.7,
            mood: MoodCategory::Energetic,
        };
        db.update_track_analysis("/a.flac", &analysis).unwrap();
        let track = db.get_track_by_path("/a.flac").unwrap();
        assert_eq!(track.bpm, Some(128.0));
        assert_eq!(track.mood, Some(MoodCategory::Energetic));
    }

    #[test]
    // 未解析トラックのみが get_unanalyzed_tracks で返されること
    fn get_unanalyzed_tracks() {
        let db = Database::new_in_memory().unwrap();
        db.insert_track(&sample_track("t1", "/a.flac")).unwrap();
        db.insert_track(&sample_track("t2", "/b.flac")).unwrap();
        let analysis = AudioAnalysis {
            bpm: 120.0, energy: 0.5, valence: 0.5, mood: MoodCategory::Mellow,
        };
        db.update_track_analysis("/a.flac", &analysis).unwrap();
        let unanalyzed = db.get_unanalyzed_tracks().unwrap();
        assert_eq!(unanalyzed.len(), 1);
        assert_eq!(unanalyzed[0].id, "t2");
    }

    #[test]
    // ムード別のトラック取得が正しく動作すること
    fn get_tracks_by_mood() {
        let db = Database::new_in_memory().unwrap();
        db.insert_track(&sample_track("t1", "/a.flac")).unwrap();
        let analysis = AudioAnalysis {
            bpm: 140.0, energy: 0.9, valence: 0.8, mood: MoodCategory::Energetic,
        };
        db.update_track_analysis("/a.flac", &analysis).unwrap();
        let tracks = db.get_tracks_by_mood(&MoodCategory::Energetic).unwrap();
        assert_eq!(tracks.len(), 1);
        let empty = db.get_tracks_by_mood(&MoodCategory::Relax).unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    // プレイリストの作成・一覧取得・削除が正しく動作すること
    fn playlist_crud() {
        let db = Database::new_in_memory().unwrap();
        let pl = db.create_playlist("My Favorites").unwrap();
        assert_eq!(pl.name, "My Favorites");
        assert!(!pl.is_smart);

        let playlists = db.get_playlists().unwrap();
        assert_eq!(playlists.len(), 1);

        db.delete_playlist(&pl.id).unwrap();
        let playlists = db.get_playlists().unwrap();
        assert!(playlists.is_empty());
    }

    #[test]
    // プレイリストへのトラック追加・削除が正しく動作すること
    fn playlist_track_management() {
        let db = Database::new_in_memory().unwrap();
        db.insert_track(&sample_track("t1", "/a.flac")).unwrap();
        db.insert_track(&sample_track("t2", "/b.flac")).unwrap();
        let pl = db.create_playlist("Test").unwrap();

        db.add_to_playlist(&pl.id, "t1").unwrap();
        db.add_to_playlist(&pl.id, "t2").unwrap();
        let tracks = db.get_playlist_tracks(&pl.id).unwrap();
        assert_eq!(tracks.len(), 2);

        db.remove_from_playlist(&pl.id, "t1").unwrap();
        let tracks = db.get_playlist_tracks(&pl.id).unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].id, "t2");
    }

    #[test]
    // 再生回数のインクリメントが正しく動作すること
    fn increment_play_count() {
        let db = Database::new_in_memory().unwrap();
        db.insert_track(&sample_track("t1", "/a.flac")).unwrap();
        db.increment_play_count("t1").unwrap();
        db.increment_play_count("t1").unwrap();
        let track = db.get_track_by_path("/a.flac").unwrap();
        assert_eq!(track.play_count, 2);
    }

    #[test]
    // レーティングの更新が正しく動作すること
    fn update_rating() {
        let db = Database::new_in_memory().unwrap();
        db.insert_track(&sample_track("t1", "/a.flac")).unwrap();
        db.update_rating("t1", 5).unwrap();
        let track = db.get_track_by_path("/a.flac").unwrap();
        assert_eq!(track.rating, 5);
    }
}
