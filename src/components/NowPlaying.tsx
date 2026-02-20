import { useEffect, useState, useCallback } from "react";
import { usePlayerStore } from "../stores/playerStore";
import { useLibraryStore } from "../stores/libraryStore";
import { formatDuration } from "../types";

interface NowPlayingProps {
  onSeek: (time: number) => void;
}

export function NowPlaying({ onSeek }: NowPlayingProps) {
  const {
    currentTrack,
    isPlaying,
    togglePlay,
    next,
    previous,
    volume,
    setVolume,
    muted,
    toggleMute,
    currentTime,
    duration,
    shuffle,
    toggleShuffle,
    repeat,
    cycleRepeat,
  } = usePlayerStore();
  const { getArtwork } = useLibraryStore();
  const [artwork, setArtwork] = useState<string | null>(null);
  const [isSeeking, setIsSeeking] = useState(false);
  const [seekValue, setSeekValue] = useState(0);

  useEffect(() => {
    if (currentTrack?.has_artwork) {
      getArtwork(currentTrack.path).then(setArtwork);
    } else {
      setArtwork(null);
    }
  }, [currentTrack?.id]);

  const handleSeekStart = () => {
    setIsSeeking(true);
    setSeekValue(currentTime);
  };

  const handleSeekChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSeekValue(parseFloat(e.target.value));
  };

  const handleSeekEnd = () => {
    onSeek(seekValue);
    setIsSeeking(false);
  };

  const progressPercent =
    duration > 0 ? ((isSeeking ? seekValue : currentTime) / duration) * 100 : 0;

  const repeatLabel = repeat === "off" ? "RPT" : repeat === "all" ? "RPT*" : "RPT1";

  return (
    <footer className="now-playing">
      {/* Track Info */}
      <div className="np-info">
        <div className="np-artwork">
          {artwork ? (
            <img src={artwork} alt="" />
          ) : (
            <div className="np-artwork-placeholder">
              <span>{currentTrack ? "//" : ""}</span>
            </div>
          )}
        </div>
        <div className="np-meta">
          {currentTrack ? (
            <>
              <span className="np-title">{currentTrack.title}</span>
              <span className="np-artist">{currentTrack.artist}</span>
            </>
          ) : (
            <span className="np-title np-empty">NO TRACK</span>
          )}
        </div>
      </div>

      {/* Playback Controls */}
      <div className="np-controls">
        <div className="np-buttons">
          <button
            className={`np-btn np-btn-sm ${shuffle ? "active" : ""}`}
            onClick={toggleShuffle}
            title="Shuffle"
          >
            SHF
          </button>
          <button className="np-btn" onClick={previous} title="Previous">
            |&lt;
          </button>
          <button
            className="np-btn np-btn-play"
            onClick={togglePlay}
            disabled={!currentTrack}
            title={isPlaying ? "Pause" : "Play"}
          >
            {isPlaying ? "||" : ">"}
          </button>
          <button className="np-btn" onClick={next} title="Next">
            &gt;|
          </button>
          <button
            className={`np-btn np-btn-sm ${repeat !== "off" ? "active" : ""}`}
            onClick={cycleRepeat}
            title="Repeat"
          >
            {repeatLabel}
          </button>
        </div>

        <div className="np-progress">
          <span className="np-time">
            {formatDuration(isSeeking ? seekValue : currentTime)}
          </span>
          <div className="np-progress-bar">
            <input
              type="range"
              min={0}
              max={duration || 0}
              step={0.1}
              value={isSeeking ? seekValue : currentTime}
              onMouseDown={handleSeekStart}
              onTouchStart={handleSeekStart}
              onChange={handleSeekChange}
              onMouseUp={handleSeekEnd}
              onTouchEnd={handleSeekEnd}
              className="np-slider progress-slider"
            />
            <div
              className="np-progress-fill"
              style={{ width: `${progressPercent}%` }}
            />
          </div>
          <span className="np-time">
            {formatDuration(duration)}
          </span>
        </div>
      </div>

      {/* Volume */}
      <div className="np-volume">
        <button
          className="np-btn np-btn-sm"
          onClick={toggleMute}
          title={muted ? "Unmute" : "Mute"}
        >
          {muted || volume === 0 ? "VOL_X" : volume < 0.5 ? "VOL_" : "VOL="}
        </button>
        <input
          type="range"
          min={0}
          max={1}
          step={0.01}
          value={muted ? 0 : volume}
          onChange={(e) => setVolume(parseFloat(e.target.value))}
          className="np-slider volume-slider"
        />
      </div>
    </footer>
  );
}
