use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub genre: String,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<u32>,
    pub duration_secs: f64,
    pub file_format: String,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub bpm: Option<f64>,
    pub energy: Option<f64>,
    pub valence: Option<f64>,
    pub mood: Option<MoodCategory>,
    pub play_count: u32,
    pub rating: u8,
    pub date_added: String,
    pub has_artwork: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MoodCategory {
    Energetic,
    Upbeat,
    Dance,
    Extreme,
    Emotional,
    Mellow,
    Relax,
    Lounge,
}

impl MoodCategory {
    pub fn all() -> Vec<MoodCategory> {
        vec![
            MoodCategory::Energetic,
            MoodCategory::Upbeat,
            MoodCategory::Dance,
            MoodCategory::Extreme,
            MoodCategory::Emotional,
            MoodCategory::Mellow,
            MoodCategory::Relax,
            MoodCategory::Lounge,
        ]
    }

    pub fn from_features(bpm: f64, energy: f64, valence: f64) -> MoodCategory {
        // SensMe-inspired mood classification
        // energy: 0.0 (calm) to 1.0 (energetic)
        // valence: 0.0 (negative/sad) to 1.0 (positive/happy)
        // bpm: tempo in beats per minute

        let high_energy = energy > 0.6;
        let high_valence = valence > 0.5;
        let fast_tempo = bpm > 120.0;

        match (high_energy, high_valence, fast_tempo) {
            (true, true, true) => MoodCategory::Energetic,
            (true, true, false) => MoodCategory::Upbeat,
            (true, false, true) => MoodCategory::Extreme,
            (true, false, false) => MoodCategory::Dance,
            (false, true, true) => MoodCategory::Dance,
            (false, true, false) => MoodCategory::Lounge,
            (false, false, true) => MoodCategory::Emotional,
            (false, false, false) => {
                if energy < 0.3 {
                    MoodCategory::Relax
                } else {
                    MoodCategory::Mellow
                }
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            MoodCategory::Energetic => "energetic",
            MoodCategory::Upbeat => "upbeat",
            MoodCategory::Dance => "dance",
            MoodCategory::Extreme => "extreme",
            MoodCategory::Emotional => "emotional",
            MoodCategory::Mellow => "mellow",
            MoodCategory::Relax => "relax",
            MoodCategory::Lounge => "lounge",
        }
    }

    pub fn from_str(s: &str) -> Option<MoodCategory> {
        match s {
            "energetic" => Some(MoodCategory::Energetic),
            "upbeat" => Some(MoodCategory::Upbeat),
            "dance" => Some(MoodCategory::Dance),
            "extreme" => Some(MoodCategory::Extreme),
            "emotional" => Some(MoodCategory::Emotional),
            "mellow" => Some(MoodCategory::Mellow),
            "relax" => Some(MoodCategory::Relax),
            "lounge" => Some(MoodCategory::Lounge),
            _ => None,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MoodCategory::Energetic => "High energy, uplifting tracks",
            MoodCategory::Upbeat => "Positive, feel-good vibes",
            MoodCategory::Dance => "Groove-driven rhythms",
            MoodCategory::Extreme => "Intense, powerful sound",
            MoodCategory::Emotional => "Deep, moving melodies",
            MoodCategory::Mellow => "Soft, gentle atmosphere",
            MoodCategory::Relax => "Calm, peaceful ambience",
            MoodCategory::Lounge => "Smooth, laid-back tunes",
        }
    }

    pub fn energy_range(&self) -> (f64, f64) {
        match self {
            MoodCategory::Energetic => (0.7, 1.0),
            MoodCategory::Upbeat => (0.5, 0.8),
            MoodCategory::Dance => (0.4, 0.7),
            MoodCategory::Extreme => (0.8, 1.0),
            MoodCategory::Emotional => (0.2, 0.5),
            MoodCategory::Mellow => (0.2, 0.5),
            MoodCategory::Relax => (0.0, 0.3),
            MoodCategory::Lounge => (0.1, 0.4),
        }
    }

    pub fn valence_range(&self) -> (f64, f64) {
        match self {
            MoodCategory::Energetic => (0.6, 1.0),
            MoodCategory::Upbeat => (0.6, 1.0),
            MoodCategory::Dance => (0.4, 0.8),
            MoodCategory::Extreme => (0.0, 0.4),
            MoodCategory::Emotional => (0.0, 0.4),
            MoodCategory::Mellow => (0.3, 0.6),
            MoodCategory::Relax => (0.4, 0.7),
            MoodCategory::Lounge => (0.5, 0.8),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub track_count: u32,
    pub is_smart: bool,
    pub mood_filter: Option<MoodCategory>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAnalysis {
    pub bpm: f64,
    pub energy: f64,
    pub valence: f64,
    pub mood: MoodCategory,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // 8種類すべてのムードカテゴリが all() で取得できること
    fn mood_category_all_returns_eight_categories() {
        let all = MoodCategory::all();
        assert_eq!(all.len(), 8);
    }

    #[test]
    // 高エネルギー・高ヴァレンス・高テンポが Energetic に分類されること
    fn from_features_energetic() {
        let mood = MoodCategory::from_features(140.0, 0.8, 0.7);
        assert_eq!(mood, MoodCategory::Energetic);
    }

    #[test]
    // 高エネルギー・高ヴァレンス・低テンポが Upbeat に分類されること
    fn from_features_upbeat() {
        let mood = MoodCategory::from_features(100.0, 0.7, 0.6);
        assert_eq!(mood, MoodCategory::Upbeat);
    }

    #[test]
    // 高エネルギー・低ヴァレンス・高テンポが Extreme に分類されること
    fn from_features_extreme() {
        let mood = MoodCategory::from_features(150.0, 0.8, 0.3);
        assert_eq!(mood, MoodCategory::Extreme);
    }

    #[test]
    // 高エネルギー・低ヴァレンス・低テンポが Dance に分類されること
    fn from_features_dance_high_energy() {
        let mood = MoodCategory::from_features(100.0, 0.7, 0.3);
        assert_eq!(mood, MoodCategory::Dance);
    }

    #[test]
    // 低エネルギー・高ヴァレンス・高テンポが Dance に分類されること
    fn from_features_dance_low_energy() {
        let mood = MoodCategory::from_features(130.0, 0.4, 0.6);
        assert_eq!(mood, MoodCategory::Dance);
    }

    #[test]
    // 低エネルギー・高ヴァレンス・低テンポが Lounge に分類されること
    fn from_features_lounge() {
        let mood = MoodCategory::from_features(90.0, 0.4, 0.6);
        assert_eq!(mood, MoodCategory::Lounge);
    }

    #[test]
    // 低エネルギー・低ヴァレンス・高テンポが Emotional に分類されること
    fn from_features_emotional() {
        let mood = MoodCategory::from_features(130.0, 0.4, 0.3);
        assert_eq!(mood, MoodCategory::Emotional);
    }

    #[test]
    // 非常に低いエネルギーが Relax に分類されること
    fn from_features_relax() {
        let mood = MoodCategory::from_features(80.0, 0.2, 0.3);
        assert_eq!(mood, MoodCategory::Relax);
    }

    #[test]
    // エネルギーが中程度（0.3以上0.6以下）で低ヴァレンス・低テンポが Mellow に分類されること
    fn from_features_mellow() {
        let mood = MoodCategory::from_features(80.0, 0.4, 0.3);
        assert_eq!(mood, MoodCategory::Mellow);
    }

    #[test]
    // as_str() と from_str() が双方向変換できること
    fn as_str_from_str_roundtrip() {
        for mood in MoodCategory::all() {
            let s = mood.as_str();
            let parsed = MoodCategory::from_str(s);
            assert_eq!(parsed, Some(mood));
        }
    }

    #[test]
    // 不正な文字列で from_str() が None を返すこと
    fn from_str_invalid_returns_none() {
        assert_eq!(MoodCategory::from_str("invalid"), None);
        assert_eq!(MoodCategory::from_str(""), None);
    }

    #[test]
    // MoodCategory の serde JSON シリアライズが snake_case であること
    fn serde_json_roundtrip() {
        let mood = MoodCategory::Energetic;
        let json = serde_json::to_string(&mood).unwrap();
        assert_eq!(json, "\"energetic\"");
        let parsed: MoodCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MoodCategory::Energetic);
    }

    #[test]
    // AudioAnalysis の JSON シリアライズ/デシリアライズが正しいこと
    fn audio_analysis_serde() {
        let analysis = AudioAnalysis {
            bpm: 128.0,
            energy: 0.75,
            valence: 0.6,
            mood: MoodCategory::Energetic,
        };
        let json = serde_json::to_string(&analysis).unwrap();
        let parsed: AudioAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.bpm, 128.0);
        assert_eq!(parsed.mood, MoodCategory::Energetic);
    }

    #[test]
    // 各カテゴリの description が空文字列でないこと
    fn description_non_empty() {
        for mood in MoodCategory::all() {
            assert!(!mood.description().is_empty());
        }
    }

    #[test]
    // energy_range の min < max で 0.0〜1.0 の範囲内であること
    fn energy_range_valid() {
        for mood in MoodCategory::all() {
            let (min, max) = mood.energy_range();
            assert!(min < max, "{:?}: {} >= {}", mood, min, max);
            assert!(min >= 0.0);
            assert!(max <= 1.0);
        }
    }

    #[test]
    // valence_range の min < max で 0.0〜1.0 の範囲内であること
    fn valence_range_valid() {
        for mood in MoodCategory::all() {
            let (min, max) = mood.valence_range();
            assert!(min < max, "{:?}: {} >= {}", mood, min, max);
            assert!(min >= 0.0);
            assert!(max <= 1.0);
        }
    }
}
