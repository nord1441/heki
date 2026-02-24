export interface Track {
  id: string;
  path: string;
  title: string;
  artist: string;
  album: string;
  album_artist: string;
  genre: string;
  track_number: number | null;
  disc_number: number | null;
  year: number | null;
  duration_secs: number;
  file_format: string;
  bitrate: number | null;
  sample_rate: number | null;
  bpm: number | null;
  energy: number | null;
  valence: number | null;
  mood: MoodCategory | null;
  play_count: number;
  rating: number;
  date_added: string;
  has_artwork: boolean;
}

export type MoodCategory =
  | "energetic"
  | "upbeat"
  | "dance"
  | "extreme"
  | "emotional"
  | "mellow"
  | "relax"
  | "lounge";

export interface Playlist {
  id: string;
  name: string;
  track_count: number;
  is_smart: boolean;
  mood_filter: MoodCategory | null;
  created_at: string;
}

export type RepeatMode = "off" | "all" | "one";

export type SortField =
  | "title"
  | "artist"
  | "album"
  | "duration_secs"
  | "track_number"
  | "year"
  | "genre"
  | "play_count"
  | "rating"
  | "date_added"
  | "bpm";

export type SortDirection = "asc" | "desc";

export interface SortConfig {
  field: SortField;
  direction: SortDirection;
}

export type ViewMode = "library" | "artist" | "album" | "genre" | "playlist" | "mood" | "queue";

export type SourceMode = "local" | "navidrome";

export interface NavidromeAuth {
  url: string;
  auth_params: string;
}

export interface MoodInfo {
  category: MoodCategory;
  label: string;
  description: string;
  icon: string;
}

export const MOOD_INFO: Record<MoodCategory, MoodInfo> = {
  energetic: {
    category: "energetic",
    label: "ENERGETIC",
    description: "High energy, uplifting tracks",
    icon: "//",
  },
  upbeat: {
    category: "upbeat",
    label: "UPBEAT",
    description: "Positive, feel-good vibes",
    icon: "/\\",
  },
  dance: {
    category: "dance",
    label: "DANCE",
    description: "Groove-driven rhythms",
    icon: "><",
  },
  extreme: {
    category: "extreme",
    label: "EXTREME",
    description: "Intense, powerful sound",
    icon: "##",
  },
  emotional: {
    category: "emotional",
    label: "EMOTIONAL",
    description: "Deep, moving melodies",
    icon: "::",
  },
  mellow: {
    category: "mellow",
    label: "MELLOW",
    description: "Soft, gentle atmosphere",
    icon: "~~",
  },
  relax: {
    category: "relax",
    label: "RELAX",
    description: "Calm, peaceful ambience",
    icon: "--",
  },
  lounge: {
    category: "lounge",
    label: "LOUNGE",
    description: "Smooth, laid-back tunes",
    icon: "..",
  },
};

export function formatDuration(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export function formatDurationLong(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  if (h > 0) {
    return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  }
  return `${m}:${s.toString().padStart(2, "0")}`;
}
