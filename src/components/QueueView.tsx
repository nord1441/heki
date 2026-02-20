import { usePlayerStore } from "../stores/playerStore";
import { formatDuration } from "../types";

export function QueueView() {
  const { queue, queueIndex, currentTrack, setQueue, removeFromQueue, clearQueue, isPlaying } =
    usePlayerStore();

  if (queue.length === 0) {
    return (
      <div className="queue-view">
        <div className="queue-header">
          <h2>PLAY QUEUE</h2>
        </div>
        <div className="track-list-empty">
          <p>QUEUE EMPTY</p>
          <p className="text-sub">Double-click a track to start playing</p>
        </div>
      </div>
    );
  }

  return (
    <div className="queue-view">
      <div className="queue-header">
        <h2>PLAY QUEUE</h2>
        <span className="text-sub">{queue.length} tracks</span>
        <button className="btn btn-secondary btn-sm" onClick={clearQueue}>
          CLEAR
        </button>
      </div>

      <div className="track-list">
        <table className="track-table">
          <thead>
            <tr>
              <th className="col-num">#</th>
              <th className="col-title">TITLE</th>
              <th className="col-artist">ARTIST</th>
              <th className="col-duration">TIME</th>
              <th className="col-actions"></th>
            </tr>
          </thead>
          <tbody>
            {queue.map((track, idx) => (
              <tr
                key={`${track.id}-${idx}`}
                className={`track-row ${idx === queueIndex ? "playing" : ""} ${
                  idx < queueIndex ? "played" : ""
                }`}
                onDoubleClick={() => setQueue(queue, idx)}
              >
                <td className="col-num">
                  {idx === queueIndex ? (
                    <span className="playing-indicator">
                      {isPlaying ? ">" : "||"}
                    </span>
                  ) : (
                    idx + 1
                  )}
                </td>
                <td className="col-title">{track.title}</td>
                <td className="col-artist">{track.artist}</td>
                <td className="col-duration">
                  {formatDuration(track.duration_secs)}
                </td>
                <td className="col-actions">
                  <button
                    className="btn-icon"
                    onClick={() => removeFromQueue(idx)}
                    title="Remove"
                  >
                    x
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
