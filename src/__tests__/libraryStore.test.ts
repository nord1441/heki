import { describe, it, expect, beforeEach } from "vitest";
import { useLibraryStore } from "../stores/libraryStore";
import type { Track } from "../types";

function makeTrack(overrides: Partial<Track> = {}): Track {
  return {
    id: "t1",
    path: "/music/track1.flac",
    title: "Track 1",
    artist: "Artist A",
    album: "Album X",
    album_artist: "",
    genre: "Rock",
    track_number: 1,
    disc_number: 1,
    year: 2024,
    duration_secs: 200,
    file_format: "FLAC",
    bitrate: 1411,
    sample_rate: 44100,
    bpm: 120,
    energy: 0.7,
    valence: 0.6,
    mood: "energetic",
    play_count: 5,
    rating: 4,
    date_added: "2024-01-01T00:00:00Z",
    has_artwork: false,
    ...overrides,
  };
}

describe("libraryStore", () => {
  beforeEach(() => {
    useLibraryStore.setState({
      tracks: [],
      filteredTracks: [],
      artists: [],
      albums: [],
      genres: [],
      playlists: [],
      selectedArtist: null,
      selectedAlbum: null,
      selectedGenre: null,
      selectedPlaylist: null,
      selectedMood: null,
      viewMode: "library",
      searchQuery: "",
      sortConfig: { field: "artist", direction: "asc" },
      isScanning: false,
      isAnalyzing: false,
      scanProgress: "",
    });
  });

  // ストアの初期状態が正しいこと
  it("has correct initial state", () => {
    const state = useLibraryStore.getState();
    expect(state.tracks).toHaveLength(0);
    expect(state.viewMode).toBe("library");
    expect(state.sortConfig).toEqual({ field: "artist", direction: "asc" });
  });

  describe("applyFilters", () => {
    const tracks = [
      makeTrack({ id: "t1", artist: "Aimer", album: "Sun Dance", genre: "Pop", title: "Song A" }),
      makeTrack({ id: "t2", artist: "Aimer", album: "Penny Rain", genre: "Rock", title: "Song B" }),
      makeTrack({ id: "t3", artist: "ZUTOMAYO", album: "Gusare", genre: "Pop", title: "Song C" }),
    ];

    // アーティストフィルターで正しく絞り込めること
    it("filters by selected artist", () => {
      useLibraryStore.setState({ tracks, selectedArtist: "Aimer" });
      useLibraryStore.getState().applyFilters();
      expect(useLibraryStore.getState().filteredTracks).toHaveLength(2);
    });

    // アルバムフィルターで正しく絞り込めること
    it("filters by selected album", () => {
      useLibraryStore.setState({ tracks, selectedAlbum: "Gusare" });
      useLibraryStore.getState().applyFilters();
      expect(useLibraryStore.getState().filteredTracks).toHaveLength(1);
      expect(useLibraryStore.getState().filteredTracks[0].id).toBe("t3");
    });

    // ジャンルフィルターで正しく絞り込めること
    it("filters by selected genre", () => {
      useLibraryStore.setState({ tracks, selectedGenre: "Rock" });
      useLibraryStore.getState().applyFilters();
      expect(useLibraryStore.getState().filteredTracks).toHaveLength(1);
    });

    // 検索クエリでタイトル・アーティスト・アルバムを横断検索できること
    it("filters by search query across title, artist, album", () => {
      useLibraryStore.setState({ tracks, searchQuery: "zuto" });
      useLibraryStore.getState().applyFilters();
      expect(useLibraryStore.getState().filteredTracks).toHaveLength(1);
      expect(useLibraryStore.getState().filteredTracks[0].artist).toBe("ZUTOMAYO");
    });

    // フィルターなしの場合は全トラックが表示されること
    it("returns all tracks when no filter is active", () => {
      useLibraryStore.setState({ tracks });
      useLibraryStore.getState().applyFilters();
      expect(useLibraryStore.getState().filteredTracks).toHaveLength(3);
    });
  });

  describe("sorting", () => {
    const tracks = [
      makeTrack({ id: "t1", title: "Zebra", artist: "C Artist", duration_secs: 300 }),
      makeTrack({ id: "t2", title: "Apple", artist: "A Artist", duration_secs: 100 }),
      makeTrack({ id: "t3", title: "Mango", artist: "B Artist", duration_secs: 200 }),
    ];

    // タイトル昇順でソートされること
    it("sorts by title ascending", () => {
      useLibraryStore.setState({
        tracks,
        sortConfig: { field: "title", direction: "asc" },
      });
      useLibraryStore.getState().applyFilters();
      const titles = useLibraryStore
        .getState()
        .filteredTracks.map((t) => t.title);
      expect(titles).toEqual(["Apple", "Mango", "Zebra"]);
    });

    // 再生時間降順でソートされること
    it("sorts by duration descending", () => {
      useLibraryStore.setState({
        tracks,
        sortConfig: { field: "duration_secs", direction: "desc" },
      });
      useLibraryStore.getState().applyFilters();
      const durations = useLibraryStore
        .getState()
        .filteredTracks.map((t) => t.duration_secs);
      expect(durations).toEqual([300, 200, 100]);
    });
  });

  describe("setViewMode", () => {
    // viewMode を library に切り替えるとフィルター選択がリセットされること
    it("resets selections when switching to library mode", () => {
      useLibraryStore.setState({
        selectedArtist: "Aimer",
        selectedAlbum: "Sun Dance",
        selectedGenre: "Pop",
        viewMode: "artist",
      });
      useLibraryStore.getState().setViewMode("library");
      const state = useLibraryStore.getState();
      expect(state.viewMode).toBe("library");
      expect(state.selectedArtist).toBeNull();
      expect(state.selectedAlbum).toBeNull();
      expect(state.selectedGenre).toBeNull();
    });
  });

  describe("setSelectedArtist / setSelectedGenre", () => {
    // setSelectedArtist が viewMode を artist に切り替えること
    it("setSelectedArtist sets viewMode to artist", () => {
      useLibraryStore.getState().setSelectedArtist("Aimer");
      expect(useLibraryStore.getState().viewMode).toBe("artist");
      expect(useLibraryStore.getState().selectedArtist).toBe("Aimer");
    });

    // setSelectedGenre が viewMode を genre に切り替えること
    it("setSelectedGenre sets viewMode to genre", () => {
      useLibraryStore.getState().setSelectedGenre("Rock");
      expect(useLibraryStore.getState().viewMode).toBe("genre");
      expect(useLibraryStore.getState().selectedGenre).toBe("Rock");
    });
  });
});
