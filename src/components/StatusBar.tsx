import { useLibraryStore } from "../stores/libraryStore";
import { useTheme } from "../hooks/useTheme";
import { formatDurationLong } from "../types";

export function StatusBar() {
  const { tracks, filteredTracks, isScanning, isAnalyzing, scanProgress } =
    useLibraryStore();
  const { theme, toggleTheme, font, toggleFont } = useTheme();

  const totalDuration = filteredTracks.reduce(
    (sum, t) => sum + t.duration_secs,
    0
  );

  return (
    <div className="status-bar">
      <div className="status-left">
        {isScanning || isAnalyzing ? (
          <span className="status-progress">
            <span className="status-spinner" />
            {scanProgress}
          </span>
        ) : (
          <span>
            {filteredTracks.length} TRACKS
            {filteredTracks.length !== tracks.length &&
              ` / ${tracks.length} TOTAL`}
            {totalDuration > 0 && ` // ${formatDurationLong(totalDuration)}`}
          </span>
        )}
      </div>
      <div className="status-right">
        <button
          className="btn-icon font-toggle"
          onClick={toggleFont}
          title={`Switch to ${font === "doto" ? "Bebas Neue" : "Doto"} font`}
        >
          {font === "doto" ? "BEBAS" : "DOTO"}
        </button>
        <span className="status-separator">/</span>
        <button
          className="btn-icon theme-toggle"
          onClick={toggleTheme}
          title={`Switch to ${theme === "dark" ? "light" : "dark"} theme`}
        >
          {theme === "dark" ? "LIGHT" : "DARK"}
        </button>
      </div>
    </div>
  );
}
