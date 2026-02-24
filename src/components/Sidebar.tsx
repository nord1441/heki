import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useLibraryStore } from "../stores/libraryStore";
import { MOOD_INFO, type MoodCategory, type ViewMode } from "../types";

export function Sidebar() {
  const {
    viewMode,
    setViewMode,
    artists,
    albums,
    genres,
    playlists,
    selectedArtist,
    selectedAlbum,
    selectedGenre,
    selectedPlaylist,
    selectedMood,
    setSelectedArtist,
    setSelectedAlbum,
    setSelectedGenre,
    setSelectedPlaylist,
    setSelectedMood,
    scanDirectory,
    analyzeAll,
    createPlaylist,
    deletePlaylist,
    isScanning,
    isAnalyzing,
    tracks,
    sourceMode,
    navidromeConnected,
    connectNavidrome,
    disconnectNavidrome,
  } = useLibraryStore();

  const [newPlaylistName, setNewPlaylistName] = useState("");
  const [showNewPlaylist, setShowNewPlaylist] = useState(false);
  const [expandedSection, setExpandedSection] = useState<string | null>(
    "library"
  );
  const [ndUrl, setNdUrl] = useState("");
  const [ndUser, setNdUser] = useState("");
  const [ndPass, setNdPass] = useState("");
  const [ndConnecting, setNdConnecting] = useState(false);
  const [ndError, setNdError] = useState("");

  const handleImport = async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      await scanDirectory(selected as string);
    }
  };

  const handleCreatePlaylist = async () => {
    if (newPlaylistName.trim()) {
      await createPlaylist(newPlaylistName.trim());
      setNewPlaylistName("");
      setShowNewPlaylist(false);
    }
  };

  const toggleSection = (section: string) => {
    setExpandedSection(expandedSection === section ? null : section);
  };

  const handleNdConnect = async () => {
    if (!ndUrl.trim() || !ndUser.trim()) return;
    setNdConnecting(true);
    setNdError("");
    try {
      await connectNavidrome(ndUrl.trim(), ndUser.trim(), ndPass);
    } catch (e) {
      setNdError(String(e));
    } finally {
      setNdConnecting(false);
    }
  };

  const handleNdDisconnect = async () => {
    await disconnectNavidrome();
    setNdUrl("");
    setNdUser("");
    setNdPass("");
    setNdError("");
  };

  const moodCategories: MoodCategory[] = [
    "energetic",
    "upbeat",
    "dance",
    "extreme",
    "emotional",
    "mellow",
    "relax",
    "lounge",
  ];

  const analyzedCount = tracks.filter((t) => t.mood !== null).length;

  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <h1 className="app-logo">heki</h1>
      </div>

      {sourceMode === "local" && (
        <div className="sidebar-actions">
          <button
            className="btn btn-primary btn-sm"
            onClick={handleImport}
            disabled={isScanning}
          >
            {isScanning ? "SCANNING..." : "+ IMPORT"}
          </button>
        </div>
      )}

      <nav className="sidebar-nav">
        {/* Server Section */}
        <div className="nav-section">
          <button
            className="nav-section-header"
            onClick={() => toggleSection("server")}
          >
            <span className="section-indicator">
              {expandedSection === "server" ? "v" : ">"}
            </span>
            <span>SERVER</span>
            {navidromeConnected && (
              <span className="server-status-badge">ON</span>
            )}
          </button>

          {expandedSection === "server" && (
            <div className="nav-items">
              {navidromeConnected ? (
                <div className="server-connected">
                  <span className="server-connected-label">NAVIDROME</span>
                  <span className="server-connected-url" title={ndUrl}>
                    {ndUrl.replace(/^https?:\/\//, "")}
                  </span>
                  <button
                    className="btn btn-secondary btn-sm nav-action-btn"
                    onClick={handleNdDisconnect}
                  >
                    DISCONNECT
                  </button>
                </div>
              ) : (
                <div className="server-form">
                  <input
                    type="text"
                    value={ndUrl}
                    onChange={(e) => setNdUrl(e.target.value)}
                    placeholder="https://navidrome.example.com"
                    className="input-sm server-input"
                    disabled={ndConnecting}
                  />
                  <input
                    type="text"
                    value={ndUser}
                    onChange={(e) => setNdUser(e.target.value)}
                    placeholder="Username"
                    className="input-sm server-input"
                    disabled={ndConnecting}
                  />
                  <input
                    type="password"
                    value={ndPass}
                    onChange={(e) => setNdPass(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && handleNdConnect()}
                    placeholder="Password"
                    className="input-sm server-input"
                    disabled={ndConnecting}
                  />
                  {ndError && <span className="server-error">{ndError}</span>}
                  <button
                    className="btn btn-primary btn-sm nav-action-btn"
                    onClick={handleNdConnect}
                    disabled={ndConnecting || !ndUrl.trim() || !ndUser.trim()}
                  >
                    {ndConnecting ? "CONNECTING..." : "CONNECT"}
                  </button>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Library Section */}
        <div className="nav-section">
          <button
            className="nav-section-header"
            onClick={() => toggleSection("library")}
          >
            <span className="section-indicator">
              {expandedSection === "library" ? "v" : ">"}
            </span>
            <span>LIBRARY</span>
            <span className="track-count">{tracks.length}</span>
          </button>

          {expandedSection === "library" && (
            <div className="nav-items">
              <button
                className={`nav-item ${viewMode === "library" ? "active" : ""}`}
                onClick={() => setViewMode("library")}
              >
                ALL TRACKS
              </button>

              <div className="nav-sub-section">
                <span className="nav-sub-label">ARTISTS</span>
                <div className="nav-sub-items">
                  {artists.map((artist) => (
                    <button
                      key={artist}
                      className={`nav-item nav-item-sub ${
                        selectedArtist === artist ? "active" : ""
                      }`}
                      onClick={() => setSelectedArtist(artist)}
                      title={artist}
                    >
                      {artist}
                    </button>
                  ))}
                </div>
              </div>

              <div className="nav-sub-section">
                <span className="nav-sub-label">ALBUMS</span>
                <div className="nav-sub-items">
                  {albums.map((album) => (
                    <button
                      key={album}
                      className={`nav-item nav-item-sub ${
                        selectedAlbum === album ? "active" : ""
                      }`}
                      onClick={() => setSelectedAlbum(album)}
                      title={album}
                    >
                      {album}
                    </button>
                  ))}
                </div>
              </div>

              <div className="nav-sub-section">
                <span className="nav-sub-label">GENRES</span>
                <div className="nav-sub-items">
                  {genres.map((genre) => (
                    <button
                      key={genre}
                      className={`nav-item nav-item-sub ${
                        selectedGenre === genre ? "active" : ""
                      }`}
                      onClick={() => setSelectedGenre(genre)}
                      title={genre}
                    >
                      {genre}
                    </button>
                  ))}
                </div>
              </div>
            </div>
          )}
        </div>

        {/* SensMe Mood Section */}
        <div className="nav-section">
          <button
            className="nav-section-header"
            onClick={() => toggleSection("mood")}
          >
            <span className="section-indicator">
              {expandedSection === "mood" ? "v" : ">"}
            </span>
            <span>SENSME</span>
            <span className="track-count">
              {analyzedCount}/{tracks.length}
            </span>
          </button>

          {expandedSection === "mood" && (
            <div className="nav-items">
              {sourceMode === "local" && (
                <button
                  className="btn btn-secondary btn-sm nav-action-btn"
                  onClick={analyzeAll}
                  disabled={isAnalyzing}
                >
                  {isAnalyzing ? "ANALYZING..." : "ANALYZE ALL"}
                </button>
              )}

              <button
                className={`nav-item ${viewMode === "mood" && !selectedMood ? "active" : ""}`}
                onClick={() => setViewMode("mood")}
              >
                MOOD MAP
              </button>

              {moodCategories.map((mood) => (
                <button
                  key={mood}
                  className={`nav-item mood-item ${
                    selectedMood === mood ? "active" : ""
                  }`}
                  onClick={() => setSelectedMood(mood)}
                >
                  <span className="mood-icon">{MOOD_INFO[mood].icon}</span>
                  <span>{MOOD_INFO[mood].label}</span>
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Playlists Section */}
        <div className="nav-section">
          <button
            className="nav-section-header"
            onClick={() => toggleSection("playlists")}
          >
            <span className="section-indicator">
              {expandedSection === "playlists" ? "v" : ">"}
            </span>
            <span>PLAYLISTS</span>
            <span className="track-count">{playlists.length}</span>
          </button>

          {expandedSection === "playlists" && (
            <div className="nav-items">
              {showNewPlaylist ? (
                <div className="new-playlist-form">
                  <input
                    type="text"
                    value={newPlaylistName}
                    onChange={(e) => setNewPlaylistName(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && handleCreatePlaylist()}
                    placeholder="Name..."
                    className="input-sm"
                    autoFocus
                  />
                  <div className="new-playlist-actions">
                    <button
                      className="btn btn-primary btn-xs"
                      onClick={handleCreatePlaylist}
                    >
                      OK
                    </button>
                    <button
                      className="btn btn-secondary btn-xs"
                      onClick={() => setShowNewPlaylist(false)}
                    >
                      X
                    </button>
                  </div>
                </div>
              ) : (
                <button
                  className="btn btn-secondary btn-sm nav-action-btn"
                  onClick={() => setShowNewPlaylist(true)}
                >
                  + NEW
                </button>
              )}

              {playlists.map((pl) => (
                <div
                  key={pl.id}
                  className={`nav-item playlist-item ${
                    selectedPlaylist === pl.id ? "active" : ""
                  }`}
                >
                  <button
                    className="playlist-name"
                    onClick={() => setSelectedPlaylist(pl.id)}
                  >
                    {pl.name}
                    <span className="playlist-count">{pl.track_count}</span>
                  </button>
                  <button
                    className="playlist-delete"
                    onClick={(e) => {
                      e.stopPropagation();
                      deletePlaylist(pl.id);
                    }}
                    title="Delete"
                  >
                    x
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Queue Section */}
        <div className="nav-section">
          <button
            className={`nav-section-header ${viewMode === "queue" ? "active" : ""}`}
            onClick={() => setViewMode("queue")}
          >
            <span className="section-indicator">&gt;</span>
            <span>QUEUE</span>
          </button>
        </div>
      </nav>
    </aside>
  );
}
