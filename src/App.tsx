import { useEffect } from "react";
import { Sidebar } from "./components/Sidebar";
import { TrackList } from "./components/TrackList";
import { NowPlaying } from "./components/NowPlaying";
import { MoodMap } from "./components/MoodMap";
import { QueueView } from "./components/QueueView";
import { SearchBar } from "./components/SearchBar";
import { StatusBar } from "./components/StatusBar";
import { useAudioPlayer } from "./hooks/useAudioPlayer";
import { useTheme } from "./hooks/useTheme";
import { useLibraryStore } from "./stores/libraryStore";
import { usePlayerStore } from "./stores/playerStore";
import { MOOD_INFO } from "./types";

export default function App() {
  const { seek } = useAudioPlayer();
  useTheme();

  const {
    filteredTracks,
    viewMode,
    selectedArtist,
    selectedAlbum,
    selectedGenre,
    selectedPlaylist,
    selectedMood,
    playlists,
    loadLibrary,
  } = useLibraryStore();

  const { queue } = usePlayerStore();

  useEffect(() => {
    loadLibrary();
  }, []);

  const getViewTitle = () => {
    switch (viewMode) {
      case "library":
        return "ALL TRACKS";
      case "artist":
        return selectedArtist || "ARTIST";
      case "album":
        return selectedAlbum || "ALBUM";
      case "genre":
        return selectedGenre || "GENRE";
      case "playlist":
        return (
          playlists.find((p) => p.id === selectedPlaylist)?.name || "PLAYLIST"
        );
      case "mood":
        return selectedMood
          ? `SENSME // ${MOOD_INFO[selectedMood].label}`
          : "SENSME CHANNELS";
      case "queue":
        return "PLAY QUEUE";
      default:
        return "LIBRARY";
    }
  };

  const renderContent = () => {
    if (viewMode === "mood" && !selectedMood) {
      return <MoodMap />;
    }

    if (viewMode === "queue") {
      return <QueueView />;
    }

    return (
      <TrackList
        tracks={filteredTracks}
        showArtist={viewMode !== "artist"}
        showAlbum={viewMode !== "album"}
        isPlaylist={viewMode === "playlist"}
        playlistId={selectedPlaylist || undefined}
      />
    );
  };

  return (
    <div className="app">
      <Sidebar />
      <main className="main-content">
        <div className="content-header">
          <h2 className="view-title">{getViewTitle()}</h2>
          <SearchBar />
        </div>

        {selectedMood && viewMode === "mood" && (
          <div className="mood-banner">
            <span className="mood-banner-icon">
              {MOOD_INFO[selectedMood].icon}
            </span>
            <div className="mood-banner-info">
              <span className="mood-banner-label">
                {MOOD_INFO[selectedMood].label}
              </span>
              <span className="mood-banner-desc">
                {MOOD_INFO[selectedMood].description}
              </span>
            </div>
            <span className="mood-banner-count">
              {filteredTracks.length} TRACKS
            </span>
          </div>
        )}

        <div className="content-body">{renderContent()}</div>
        <StatusBar />
      </main>
      <NowPlaying onSeek={seek} />
    </div>
  );
}
