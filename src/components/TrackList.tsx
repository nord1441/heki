import { useCallback, useRef, useState } from "react";
import { usePlayerStore } from "../stores/playerStore";
import { useLibraryStore } from "../stores/libraryStore";
import {
  formatDuration,
  type SortField,
  type Track,
} from "../types";

interface TrackListProps {
  tracks: Track[];
  showAlbum?: boolean;
  showArtist?: boolean;
  isPlaylist?: boolean;
  playlistId?: string;
}

export function TrackList({
  tracks,
  showAlbum = true,
  showArtist = true,
  isPlaylist = false,
  playlistId,
}: TrackListProps) {
  const { currentTrack, setQueue, isPlaying, togglePlay } = usePlayerStore();
  const { sortConfig, setSortConfig, removeFromPlaylist, addToPlaylist, playlists } =
    useLibraryStore();
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    track: Track;
  } | null>(null);
  const [selectedTrackIds, setSelectedTrackIds] = useState<Set<string>>(
    new Set()
  );
  const lastClickedRef = useRef<string | null>(null);

  const handleDoubleClick = useCallback(
    (track: Track, index: number) => {
      setQueue(tracks, index);
      if (!isPlaying) togglePlay();
    },
    [tracks, setQueue, isPlaying, togglePlay]
  );

  const handleClick = useCallback(
    (track: Track, e: React.MouseEvent) => {
      if (e.shiftKey && lastClickedRef.current) {
        const lastIdx = tracks.findIndex((t) => t.id === lastClickedRef.current);
        const curIdx = tracks.findIndex((t) => t.id === track.id);
        const [start, end] = [
          Math.min(lastIdx, curIdx),
          Math.max(lastIdx, curIdx),
        ];
        const range = tracks.slice(start, end + 1).map((t) => t.id);
        setSelectedTrackIds(new Set(range));
      } else if (e.ctrlKey || e.metaKey) {
        setSelectedTrackIds((prev) => {
          const next = new Set(prev);
          if (next.has(track.id)) {
            next.delete(track.id);
          } else {
            next.add(track.id);
          }
          return next;
        });
      } else {
        setSelectedTrackIds(new Set([track.id]));
      }
      lastClickedRef.current = track.id;
    },
    [tracks]
  );

  const handleSort = (field: SortField) => {
    if (sortConfig.field === field) {
      setSortConfig({
        field,
        direction: sortConfig.direction === "asc" ? "desc" : "asc",
      });
    } else {
      setSortConfig({ field, direction: "asc" });
    }
  };

  const handleContextMenu = (e: React.MouseEvent, track: Track) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, track });
  };

  const closeContextMenu = () => setContextMenu(null);

  const sortIndicator = (field: SortField) => {
    if (sortConfig.field !== field) return "";
    return sortConfig.direction === "asc" ? " ^" : " v";
  };

  return (
    <div className="track-list" onClick={() => contextMenu && closeContextMenu()}>
      <table className="track-table">
        <thead>
          <tr>
            <th className="col-num" onClick={() => handleSort("track_number")}>
              #<span className="sort-ind">{sortIndicator("track_number")}</span>
            </th>
            <th className="col-title" onClick={() => handleSort("title")}>
              TITLE
              <span className="sort-ind">{sortIndicator("title")}</span>
            </th>
            {showArtist && (
              <th className="col-artist" onClick={() => handleSort("artist")}>
                ARTIST
                <span className="sort-ind">{sortIndicator("artist")}</span>
              </th>
            )}
            {showAlbum && (
              <th className="col-album" onClick={() => handleSort("album")}>
                ALBUM
                <span className="sort-ind">{sortIndicator("album")}</span>
              </th>
            )}
            <th className="col-genre" onClick={() => handleSort("genre")}>
              GENRE
              <span className="sort-ind">{sortIndicator("genre")}</span>
            </th>
            <th className="col-year" onClick={() => handleSort("year")}>
              YEAR
              <span className="sort-ind">{sortIndicator("year")}</span>
            </th>
            <th className="col-duration" onClick={() => handleSort("duration_secs")}>
              TIME
              <span className="sort-ind">{sortIndicator("duration_secs")}</span>
            </th>
            <th className="col-bpm" onClick={() => handleSort("bpm")}>
              BPM
              <span className="sort-ind">{sortIndicator("bpm")}</span>
            </th>
            <th className="col-plays" onClick={() => handleSort("play_count")}>
              PLAYS
              <span className="sort-ind">{sortIndicator("play_count")}</span>
            </th>
          </tr>
        </thead>
        <tbody>
          {tracks.map((track, idx) => (
            <tr
              key={track.id}
              className={`track-row ${
                currentTrack?.id === track.id ? "playing" : ""
              } ${selectedTrackIds.has(track.id) ? "selected" : ""}`}
              onClick={(e) => handleClick(track, e)}
              onDoubleClick={() => handleDoubleClick(track, idx)}
              onContextMenu={(e) => handleContextMenu(e, track)}
            >
              <td className="col-num">
                {currentTrack?.id === track.id ? (
                  <span className="playing-indicator">
                    {isPlaying ? ">" : "||"}
                  </span>
                ) : (
                  track.track_number || ""
                )}
              </td>
              <td className="col-title" title={track.title}>
                {track.title}
              </td>
              {showArtist && (
                <td className="col-artist" title={track.artist}>
                  {track.artist}
                </td>
              )}
              {showAlbum && (
                <td className="col-album" title={track.album}>
                  {track.album}
                </td>
              )}
              <td className="col-genre">{track.genre}</td>
              <td className="col-year">{track.year || ""}</td>
              <td className="col-duration">
                {formatDuration(track.duration_secs)}
              </td>
              <td className="col-bpm">
                {track.bpm ? Math.round(track.bpm) : ""}
              </td>
              <td className="col-plays">{track.play_count || ""}</td>
            </tr>
          ))}
        </tbody>
      </table>

      {tracks.length === 0 && (
        <div className="track-list-empty">
          <p>NO TRACKS</p>
          <p className="text-sub">Import a folder to get started</p>
        </div>
      )}

      {contextMenu && (
        <div
          className="context-menu"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onClick={(e) => e.stopPropagation()}
        >
          <button
            className="context-item"
            onClick={() => {
              const idx = tracks.findIndex(
                (t) => t.id === contextMenu.track.id
              );
              setQueue(tracks, idx);
              closeContextMenu();
            }}
          >
            PLAY NOW
          </button>
          <button
            className="context-item"
            onClick={() => {
              usePlayerStore.getState().addToQueue([contextMenu.track]);
              closeContextMenu();
            }}
          >
            ADD TO QUEUE
          </button>
          <div className="context-separator" />
          {playlists.map((pl) => (
            <button
              key={pl.id}
              className="context-item"
              onClick={() => {
                addToPlaylist(pl.id, contextMenu.track.id);
                closeContextMenu();
              }}
            >
              + {pl.name}
            </button>
          ))}
          {isPlaylist && playlistId && (
            <>
              <div className="context-separator" />
              <button
                className="context-item context-item-danger"
                onClick={() => {
                  removeFromPlaylist(playlistId, contextMenu.track.id);
                  closeContextMenu();
                }}
              >
                REMOVE
              </button>
            </>
          )}
        </div>
      )}
    </div>
  );
}
