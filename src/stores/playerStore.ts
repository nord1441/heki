import { create } from "zustand";
import type { RepeatMode, Track } from "../types";

interface PlayerState {
  currentTrack: Track | null;
  queue: Track[];
  queueIndex: number;
  isPlaying: boolean;
  volume: number;
  muted: boolean;
  currentTime: number;
  duration: number;
  shuffle: boolean;
  repeat: RepeatMode;
  shuffleHistory: number[];

  setCurrentTrack: (track: Track | null) => void;
  setQueue: (tracks: Track[], startIndex?: number) => void;
  addToQueue: (tracks: Track[]) => void;
  removeFromQueue: (index: number) => void;
  clearQueue: () => void;
  play: () => void;
  pause: () => void;
  togglePlay: () => void;
  next: () => void;
  previous: () => void;
  setVolume: (volume: number) => void;
  toggleMute: () => void;
  setCurrentTime: (time: number) => void;
  setDuration: (duration: number) => void;
  toggleShuffle: () => void;
  cycleRepeat: () => void;
}

export const usePlayerStore = create<PlayerState>((set, get) => ({
  currentTrack: null,
  queue: [],
  queueIndex: -1,
  isPlaying: false,
  volume: 0.8,
  muted: false,
  currentTime: 0,
  duration: 0,
  shuffle: false,
  repeat: "off",
  shuffleHistory: [],

  setCurrentTrack: (track) => set({ currentTrack: track }),

  setQueue: (tracks, startIndex = 0) =>
    set({
      queue: tracks,
      queueIndex: startIndex,
      currentTrack: tracks[startIndex] || null,
      shuffleHistory: [startIndex],
    }),

  addToQueue: (tracks) =>
    set((state) => ({ queue: [...state.queue, ...tracks] })),

  removeFromQueue: (index) =>
    set((state) => {
      const newQueue = [...state.queue];
      newQueue.splice(index, 1);
      let newIndex = state.queueIndex;
      if (index < newIndex) newIndex--;
      if (index === newIndex) {
        return {
          queue: newQueue,
          queueIndex: newIndex,
          currentTrack: newQueue[newIndex] || null,
        };
      }
      return { queue: newQueue, queueIndex: newIndex };
    }),

  clearQueue: () =>
    set({ queue: [], queueIndex: -1, currentTrack: null, isPlaying: false }),

  play: () => set({ isPlaying: true }),
  pause: () => set({ isPlaying: false }),
  togglePlay: () => set((state) => ({ isPlaying: !state.isPlaying })),

  next: () => {
    const { queue, queueIndex, shuffle, repeat, shuffleHistory } = get();
    if (queue.length === 0) return;

    let nextIndex: number;

    if (repeat === "one") {
      nextIndex = queueIndex;
    } else if (shuffle) {
      const unplayed = queue
        .map((_, i) => i)
        .filter((i) => !shuffleHistory.includes(i));
      if (unplayed.length === 0) {
        if (repeat === "all") {
          nextIndex = Math.floor(Math.random() * queue.length);
          set({ shuffleHistory: [nextIndex] });
        } else {
          set({ isPlaying: false });
          return;
        }
      } else {
        nextIndex = unplayed[Math.floor(Math.random() * unplayed.length)];
        set({
          shuffleHistory: [...shuffleHistory, nextIndex],
        });
      }
    } else {
      nextIndex = queueIndex + 1;
      if (nextIndex >= queue.length) {
        if (repeat === "all") {
          nextIndex = 0;
        } else {
          set({ isPlaying: false });
          return;
        }
      }
    }

    set({
      queueIndex: nextIndex,
      currentTrack: queue[nextIndex],
      currentTime: 0,
    });
  },

  previous: () => {
    const { queue, queueIndex, currentTime, repeat } = get();
    if (queue.length === 0) return;

    // If more than 3 seconds in, restart current track
    if (currentTime > 3) {
      set({ currentTime: 0 });
      return;
    }

    let prevIndex = queueIndex - 1;
    if (prevIndex < 0) {
      prevIndex = repeat === "all" ? queue.length - 1 : 0;
    }

    set({
      queueIndex: prevIndex,
      currentTrack: queue[prevIndex],
      currentTime: 0,
    });
  },

  setVolume: (volume) => set({ volume, muted: volume === 0 }),
  toggleMute: () =>
    set((state) => ({ muted: !state.muted })),
  setCurrentTime: (time) => set({ currentTime: time }),
  setDuration: (duration) => set({ duration }),
  toggleShuffle: () => set((state) => ({ shuffle: !state.shuffle })),
  cycleRepeat: () =>
    set((state) => {
      const modes: RepeatMode[] = ["off", "all", "one"];
      const idx = modes.indexOf(state.repeat);
      return { repeat: modes[(idx + 1) % 3] };
    }),
}));
