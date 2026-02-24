import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type {
  MoodCategory,
  NavidromeAuth,
  Playlist,
  SortConfig,
  SourceMode,
  Track,
  ViewMode,
} from "../types";

interface LibraryState {
  tracks: Track[];
  filteredTracks: Track[];
  artists: string[];
  albums: string[];
  genres: string[];
  playlists: Playlist[];
  selectedArtist: string | null;
  selectedAlbum: string | null;
  selectedGenre: string | null;
  selectedPlaylist: string | null;
  selectedMood: MoodCategory | null;
  viewMode: ViewMode;
  searchQuery: string;
  sortConfig: SortConfig;
  isScanning: boolean;
  isAnalyzing: boolean;
  scanProgress: string;

  sourceMode: SourceMode;
  navidromeAuth: NavidromeAuth | null;
  navidromeConnected: boolean;

  loadLibrary: () => Promise<void>;
  scanDirectory: (path: string) => Promise<void>;
  analyzeAll: () => Promise<void>;
  setViewMode: (mode: ViewMode) => void;
  setSelectedArtist: (artist: string | null) => void;
  setSelectedAlbum: (album: string | null) => void;
  setSelectedGenre: (genre: string | null) => void;
  setSelectedPlaylist: (id: string | null) => void;
  setSelectedMood: (mood: MoodCategory | null) => void;
  setSearchQuery: (query: string) => void;
  setSortConfig: (config: SortConfig) => void;
  applyFilters: () => void;

  createPlaylist: (name: string) => Promise<void>;
  deletePlaylist: (id: string) => Promise<void>;
  addToPlaylist: (playlistId: string, trackId: string) => Promise<void>;
  removeFromPlaylist: (playlistId: string, trackId: string) => Promise<void>;
  loadPlaylists: () => Promise<void>;
  loadPlaylistTracks: (playlistId: string) => Promise<Track[]>;

  incrementPlayCount: (trackId: string) => Promise<void>;
  updateRating: (trackId: string, rating: number) => Promise<void>;
  getArtwork: (path: string) => Promise<string | null>;

  connectNavidrome: (
    url: string,
    username: string,
    password: string
  ) => Promise<void>;
  disconnectNavidrome: () => Promise<void>;
}

export const useLibraryStore = create<LibraryState>((set, get) => ({
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

  sourceMode: "local",
  navidromeAuth: null,
  navidromeConnected: false,

  connectNavidrome: async (url, username, password) => {
    try {
      const auth = await invoke<NavidromeAuth>("nd_connect", {
        url,
        username,
        password,
      });
      set({
        sourceMode: "navidrome",
        navidromeAuth: auth,
        navidromeConnected: true,
        // Reset library state
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
      });
      await get().loadLibrary();
    } catch (e) {
      throw e;
    }
  },

  disconnectNavidrome: async () => {
    try {
      await invoke("nd_disconnect");
    } catch {
      // Ignore disconnect errors
    }
    set({
      sourceMode: "local",
      navidromeAuth: null,
      navidromeConnected: false,
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
    });
    await get().loadLibrary();
  },

  loadLibrary: async () => {
    const { sourceMode } = get();
    try {
      if (sourceMode === "navidrome") {
        const [tracks, artists, albums, genres, playlists] = await Promise.all([
          invoke<Track[]>("nd_get_all_tracks"),
          invoke<string[]>("nd_get_artists"),
          invoke<string[]>("nd_get_albums"),
          invoke<string[]>("nd_get_genres"),
          invoke<Playlist[]>("nd_get_playlists"),
        ]);
        set({ tracks, artists, albums, genres, playlists });
      } else {
        const [tracks, artists, albums, genres, playlists] = await Promise.all([
          invoke<Track[]>("get_all_tracks"),
          invoke<string[]>("get_artists"),
          invoke<string[]>("get_albums"),
          invoke<string[]>("get_genres"),
          invoke<Playlist[]>("get_playlists"),
        ]);
        set({ tracks, artists, albums, genres, playlists });
      }
      get().applyFilters();
    } catch (e) {
      console.error("Failed to load library:", e);
    }
  },

  scanDirectory: async (path: string) => {
    set({ isScanning: true, scanProgress: "Scanning..." });
    try {
      await invoke<Track[]>("scan_directory", { path });
      await get().loadLibrary();
    } catch (e) {
      console.error("Scan failed:", e);
    } finally {
      set({ isScanning: false, scanProgress: "" });
    }
  },

  analyzeAll: async () => {
    set({ isAnalyzing: true, scanProgress: "Analyzing mood & tempo..." });
    try {
      await invoke<Track[]>("analyze_all_tracks");
      await get().loadLibrary();
    } catch (e) {
      console.error("Analysis failed:", e);
    } finally {
      set({ isAnalyzing: false, scanProgress: "" });
    }
  },

  setViewMode: (mode) => {
    set({ viewMode: mode });
    if (mode === "library") {
      set({
        selectedArtist: null,
        selectedAlbum: null,
        selectedGenre: null,
        selectedPlaylist: null,
        selectedMood: null,
      });
    }
    get().applyFilters();
  },

  setSelectedArtist: (artist) => {
    set({ selectedArtist: artist, selectedAlbum: null, viewMode: "artist" });
    get().applyFilters();
  },

  setSelectedAlbum: (album) => {
    set({ selectedAlbum: album, viewMode: "album" });
    get().applyFilters();
  },

  setSelectedGenre: (genre) => {
    set({ selectedGenre: genre, viewMode: "genre" });
    get().applyFilters();
  },

  setSelectedPlaylist: async (id) => {
    const { sourceMode } = get();
    set({ selectedPlaylist: id, viewMode: "playlist" });
    if (id) {
      try {
        const cmd =
          sourceMode === "navidrome"
            ? "nd_get_playlist_tracks"
            : "get_playlist_tracks";
        const tracks = await invoke<Track[]>(cmd, { playlistId: id });
        set({ filteredTracks: tracks });
      } catch (e) {
        console.error("Failed to load playlist tracks:", e);
      }
    }
  },

  setSelectedMood: async (mood) => {
    set({ selectedMood: mood, viewMode: "mood" });
    if (mood) {
      try {
        const tracks = await invoke<Track[]>("get_tracks_by_mood", { mood });
        set({ filteredTracks: tracks });
      } catch (e) {
        console.error("Failed to load mood tracks:", e);
      }
    }
  },

  setSearchQuery: (query) => {
    set({ searchQuery: query });
    get().applyFilters();
  },

  setSortConfig: (config) => {
    set({ sortConfig: config });
    get().applyFilters();
  },

  applyFilters: () => {
    const {
      tracks,
      selectedArtist,
      selectedAlbum,
      selectedGenre,
      searchQuery,
      sortConfig,
      viewMode,
    } = get();

    let filtered = [...tracks];

    if (selectedArtist) {
      filtered = filtered.filter((t) => t.artist === selectedArtist);
    }
    if (selectedAlbum) {
      filtered = filtered.filter((t) => t.album === selectedAlbum);
    }
    if (selectedGenre) {
      filtered = filtered.filter((t) => t.genre === selectedGenre);
    }

    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (t) =>
          t.title.toLowerCase().includes(q) ||
          t.artist.toLowerCase().includes(q) ||
          t.album.toLowerCase().includes(q)
      );
    }

    // Sort
    filtered.sort((a, b) => {
      const field = sortConfig.field;
      let aVal = a[field];
      let bVal = b[field];

      if (aVal == null) aVal = "";
      if (bVal == null) bVal = "";

      if (typeof aVal === "string" && typeof bVal === "string") {
        const cmp = aVal.localeCompare(bVal);
        return sortConfig.direction === "asc" ? cmp : -cmp;
      }
      if (typeof aVal === "number" && typeof bVal === "number") {
        return sortConfig.direction === "asc" ? aVal - bVal : bVal - aVal;
      }
      return 0;
    });

    if (viewMode !== "playlist" && viewMode !== "mood") {
      set({ filteredTracks: filtered });
    }
  },

  createPlaylist: async (name) => {
    const { sourceMode } = get();
    try {
      const cmd =
        sourceMode === "navidrome" ? "nd_create_playlist" : "create_playlist";
      await invoke(cmd, { name });
      await get().loadPlaylists();
    } catch (e) {
      console.error("Failed to create playlist:", e);
    }
  },

  deletePlaylist: async (id) => {
    const { sourceMode } = get();
    try {
      const cmd =
        sourceMode === "navidrome" ? "nd_delete_playlist" : "delete_playlist";
      await invoke(cmd, { playlistId: id });
      await get().loadPlaylists();
    } catch (e) {
      console.error("Failed to delete playlist:", e);
    }
  },

  addToPlaylist: async (playlistId, trackId) => {
    const { sourceMode } = get();
    try {
      const cmd =
        sourceMode === "navidrome" ? "nd_add_to_playlist" : "add_to_playlist";
      await invoke(cmd, { playlistId, trackId });
      await get().loadPlaylists();
    } catch (e) {
      console.error("Failed to add to playlist:", e);
    }
  },

  removeFromPlaylist: async (playlistId, trackId) => {
    const { sourceMode } = get();
    try {
      const cmd =
        sourceMode === "navidrome"
          ? "nd_remove_from_playlist"
          : "remove_from_playlist";
      await invoke(cmd, { playlistId, trackId });
      if (get().selectedPlaylist === playlistId) {
        await get().setSelectedPlaylist(playlistId);
      }
    } catch (e) {
      console.error("Failed to remove from playlist:", e);
    }
  },

  loadPlaylists: async () => {
    const { sourceMode } = get();
    try {
      const cmd =
        sourceMode === "navidrome" ? "nd_get_playlists" : "get_playlists";
      const playlists = await invoke<Playlist[]>(cmd);
      set({ playlists });
    } catch (e) {
      console.error("Failed to load playlists:", e);
    }
  },

  loadPlaylistTracks: async (playlistId) => {
    const { sourceMode } = get();
    try {
      const cmd =
        sourceMode === "navidrome"
          ? "nd_get_playlist_tracks"
          : "get_playlist_tracks";
      return await invoke<Track[]>(cmd, { playlistId });
    } catch {
      return [];
    }
  },

  incrementPlayCount: async (trackId) => {
    const { sourceMode } = get();
    try {
      if (sourceMode === "navidrome") {
        await invoke("nd_scrobble", { trackId });
      } else {
        await invoke("increment_play_count", { trackId });
      }
    } catch (e) {
      console.error("Failed to increment play count:", e);
    }
  },

  updateRating: async (trackId, rating) => {
    const { sourceMode } = get();
    try {
      if (sourceMode === "navidrome") {
        await invoke("nd_set_rating", { trackId, rating });
      } else {
        await invoke("update_rating", { trackId, rating });
      }
      set((state) => ({
        tracks: state.tracks.map((t) =>
          t.id === trackId ? { ...t, rating } : t
        ),
        filteredTracks: state.filteredTracks.map((t) =>
          t.id === trackId ? { ...t, rating } : t
        ),
      }));
    } catch (e) {
      console.error("Failed to update rating:", e);
    }
  },

  getArtwork: async (path) => {
    const { sourceMode, navidromeAuth } = get();
    if (sourceMode === "navidrome" && path.startsWith("navidrome://")) {
      if (!navidromeAuth) return null;
      const songId = path.replace("navidrome://", "");
      return `${navidromeAuth.url}/rest/getCoverArt?id=${songId}&size=300&${navidromeAuth.auth_params}`;
    }
    try {
      return await invoke<string | null>("get_track_artwork", { path });
    } catch {
      return null;
    }
  },
}));
