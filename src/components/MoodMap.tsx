import { useMemo } from "react";
import { useLibraryStore } from "../stores/libraryStore";
import { usePlayerStore } from "../stores/playerStore";
import { MOOD_INFO, type MoodCategory, type Track } from "../types";

export function MoodMap() {
  const { tracks, setSelectedMood } = useLibraryStore();
  const { setQueue } = usePlayerStore();

  const analyzedTracks = useMemo(
    () => tracks.filter((t) => t.mood && t.energy !== null && t.valence !== null),
    [tracks]
  );

  const moodGroups = useMemo(() => {
    const groups: Record<MoodCategory, Track[]> = {
      energetic: [],
      upbeat: [],
      dance: [],
      extreme: [],
      emotional: [],
      mellow: [],
      relax: [],
      lounge: [],
    };
    for (const track of analyzedTracks) {
      if (track.mood) {
        groups[track.mood].push(track);
      }
    }
    return groups;
  }, [analyzedTracks]);

  // Position mood categories on a 2D map (energy vs valence)
  const moodPositions: Record<MoodCategory, { x: number; y: number }> = {
    energetic: { x: 75, y: 15 },
    upbeat: { x: 75, y: 35 },
    dance: { x: 55, y: 25 },
    extreme: { x: 25, y: 15 },
    emotional: { x: 25, y: 55 },
    mellow: { x: 40, y: 70 },
    relax: { x: 60, y: 75 },
    lounge: { x: 70, y: 60 },
  };

  const handleMoodClick = (mood: MoodCategory) => {
    setSelectedMood(mood);
  };

  const handlePlayMood = (mood: MoodCategory, e: React.MouseEvent) => {
    e.stopPropagation();
    const moodTracks = moodGroups[mood];
    if (moodTracks.length > 0) {
      setQueue(moodTracks, 0);
    }
  };

  return (
    <div className="mood-map-container">
      <div className="mood-map-header">
        <h2>SENSME CHANNELS</h2>
        <p className="text-sub">
          {analyzedTracks.length} / {tracks.length} TRACKS ANALYZED
        </p>
      </div>

      <div className="mood-map">
        {/* Axis labels */}
        <div className="mood-axis mood-axis-y-top">ENERGETIC</div>
        <div className="mood-axis mood-axis-y-bottom">CALM</div>
        <div className="mood-axis mood-axis-x-left">DARK</div>
        <div className="mood-axis mood-axis-x-right">BRIGHT</div>

        {/* Grid lines */}
        <div className="mood-grid">
          <div className="mood-grid-line mood-grid-h" />
          <div className="mood-grid-line mood-grid-v" />
        </div>

        {/* Mood nodes */}
        {(Object.entries(moodPositions) as [MoodCategory, { x: number; y: number }][]).map(
          ([mood, pos]) => {
            const count = moodGroups[mood].length;
            const size = Math.max(48, Math.min(96, 48 + count * 2));
            return (
              <div
                key={mood}
                className={`mood-node ${count > 0 ? "has-tracks" : ""}`}
                style={{
                  left: `${pos.x}%`,
                  top: `${pos.y}%`,
                  width: `${size}px`,
                  height: `${size}px`,
                }}
                onClick={() => handleMoodClick(mood)}
              >
                <span className="mood-node-icon">{MOOD_INFO[mood].icon}</span>
                <span className="mood-node-label">{MOOD_INFO[mood].label}</span>
                <span className="mood-node-count">{count}</span>
                {count > 0 && (
                  <button
                    className="mood-node-play"
                    onClick={(e) => handlePlayMood(mood, e)}
                  >
                    &gt;
                  </button>
                )}
              </div>
            );
          }
        )}

        {/* Individual track dots */}
        {analyzedTracks.map((track) => (
          <div
            key={track.id}
            className="mood-dot"
            style={{
              left: `${(track.valence || 0.5) * 100}%`,
              bottom: `${(track.energy || 0.5) * 100}%`,
            }}
            title={`${track.title} - ${track.artist}`}
          />
        ))}
      </div>
    </div>
  );
}
