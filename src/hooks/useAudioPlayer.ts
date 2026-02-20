import { useEffect, useRef, useCallback } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { usePlayerStore } from "../stores/playerStore";
import { useLibraryStore } from "../stores/libraryStore";

export function useAudioPlayer() {
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const {
    currentTrack,
    isPlaying,
    volume,
    muted,
    currentTime,
    next,
    setCurrentTime,
    setDuration,
    pause,
  } = usePlayerStore();
  const { incrementPlayCount } = useLibraryStore();
  const hasCountedRef = useRef(false);

  // Initialize audio element
  useEffect(() => {
    if (!audioRef.current) {
      audioRef.current = new Audio();
      audioRef.current.preload = "auto";
    }

    const audio = audioRef.current;

    const onTimeUpdate = () => {
      usePlayerStore.getState().setCurrentTime(audio.currentTime);
    };

    const onDurationChange = () => {
      if (audio.duration && !isNaN(audio.duration)) {
        setDuration(audio.duration);
      }
    };

    const onEnded = () => {
      const state = usePlayerStore.getState();
      if (state.repeat === "one") {
        audio.currentTime = 0;
        audio.play();
      } else {
        state.next();
      }
    };

    const onError = (e: Event) => {
      console.error("Audio playback error:", e);
      pause();
    };

    audio.addEventListener("timeupdate", onTimeUpdate);
    audio.addEventListener("durationchange", onDurationChange);
    audio.addEventListener("ended", onEnded);
    audio.addEventListener("error", onError);

    return () => {
      audio.removeEventListener("timeupdate", onTimeUpdate);
      audio.removeEventListener("durationchange", onDurationChange);
      audio.removeEventListener("ended", onEnded);
      audio.removeEventListener("error", onError);
    };
  }, []);

  // Load new track
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio || !currentTrack) return;

    const src = convertFileSrc(currentTrack.path);
    audio.src = src;
    audio.load();
    hasCountedRef.current = false;

    if (isPlaying) {
      audio.play().catch(console.error);
    }
  }, [currentTrack?.id]);

  // Play/pause
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio || !currentTrack) return;

    if (isPlaying) {
      audio.play().catch(console.error);
    } else {
      audio.pause();
    }
  }, [isPlaying]);

  // Volume
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.volume = muted ? 0 : volume;
  }, [volume, muted]);

  // Increment play count after 30 seconds or 50%
  useEffect(() => {
    if (
      !hasCountedRef.current &&
      currentTrack &&
      currentTime > 0
    ) {
      const threshold = Math.min(30, currentTrack.duration_secs * 0.5);
      if (currentTime >= threshold) {
        hasCountedRef.current = true;
        incrementPlayCount(currentTrack.id);
      }
    }
  }, [currentTime, currentTrack]);

  const seek = useCallback((time: number) => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.currentTime = time;
    setCurrentTime(time);
  }, []);

  return { audioRef, seek };
}
