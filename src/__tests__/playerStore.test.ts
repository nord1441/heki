import { describe, it, expect, beforeEach } from "vitest";
import { usePlayerStore } from "../stores/playerStore";
import type { Track } from "../types";

/** テスト用のダミートラックを生成する */
function makeTrack(overrides: Partial<Track> = {}): Track {
  return {
    id: "t1",
    path: "/music/track1.flac",
    title: "Track 1",
    artist: "Artist",
    album: "Album",
    album_artist: "",
    genre: "Rock",
    track_number: 1,
    disc_number: 1,
    year: 2024,
    duration_secs: 240,
    file_format: "FLAC",
    bitrate: 1411,
    sample_rate: 44100,
    bpm: null,
    energy: null,
    valence: null,
    mood: null,
    play_count: 0,
    rating: 0,
    date_added: "2024-01-01T00:00:00Z",
    has_artwork: false,
    ...overrides,
  };
}

function makeTracks(n: number): Track[] {
  return Array.from({ length: n }, (_, i) =>
    makeTrack({ id: `t${i + 1}`, title: `Track ${i + 1}` })
  );
}

describe("playerStore", () => {
  beforeEach(() => {
    // 各テスト前にストアを初期状態にリセットする
    usePlayerStore.setState({
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
    });
  });

  // ストアの初期状態が正しいこと
  it("has correct initial state", () => {
    const state = usePlayerStore.getState();
    expect(state.currentTrack).toBeNull();
    expect(state.queue).toHaveLength(0);
    expect(state.isPlaying).toBe(false);
    expect(state.volume).toBe(0.8);
    expect(state.repeat).toBe("off");
    expect(state.shuffle).toBe(false);
  });

  describe("setQueue", () => {
    // キューを設定すると currentTrack が指定インデックスのトラックになること
    it("sets queue and selects track at given index", () => {
      const tracks = makeTracks(3);
      usePlayerStore.getState().setQueue(tracks, 1);
      const state = usePlayerStore.getState();
      expect(state.queue).toHaveLength(3);
      expect(state.queueIndex).toBe(1);
      expect(state.currentTrack?.id).toBe("t2");
    });

    // startIndex を省略した場合、先頭トラックが選択されること
    it("defaults to index 0 when startIndex omitted", () => {
      const tracks = makeTracks(2);
      usePlayerStore.getState().setQueue(tracks);
      expect(usePlayerStore.getState().currentTrack?.id).toBe("t1");
    });
  });

  describe("play / pause / togglePlay", () => {
    // play() で isPlaying が true になること
    it("play sets isPlaying to true", () => {
      usePlayerStore.getState().play();
      expect(usePlayerStore.getState().isPlaying).toBe(true);
    });

    // pause() で isPlaying が false になること
    it("pause sets isPlaying to false", () => {
      usePlayerStore.getState().play();
      usePlayerStore.getState().pause();
      expect(usePlayerStore.getState().isPlaying).toBe(false);
    });

    // togglePlay() で isPlaying がトグルされること
    it("togglePlay toggles isPlaying", () => {
      usePlayerStore.getState().togglePlay();
      expect(usePlayerStore.getState().isPlaying).toBe(true);
      usePlayerStore.getState().togglePlay();
      expect(usePlayerStore.getState().isPlaying).toBe(false);
    });
  });

  describe("next", () => {
    // 次のトラックに進むこと
    it("advances to the next track", () => {
      const tracks = makeTracks(3);
      usePlayerStore.getState().setQueue(tracks, 0);
      usePlayerStore.getState().next();
      expect(usePlayerStore.getState().currentTrack?.id).toBe("t2");
      expect(usePlayerStore.getState().currentTime).toBe(0);
    });

    // repeat=off でキュー末尾に達した場合、再生が停止すること
    it("stops at end of queue when repeat is off", () => {
      const tracks = makeTracks(2);
      usePlayerStore.getState().setQueue(tracks, 1);
      usePlayerStore.getState().play();
      usePlayerStore.getState().next();
      expect(usePlayerStore.getState().isPlaying).toBe(false);
    });

    // repeat=all でキュー末尾の場合、先頭に戻ること
    it("wraps to beginning when repeat is all", () => {
      const tracks = makeTracks(3);
      usePlayerStore.getState().setQueue(tracks, 2);
      usePlayerStore.setState({ repeat: "all" });
      usePlayerStore.getState().next();
      expect(usePlayerStore.getState().currentTrack?.id).toBe("t1");
    });

    // repeat=one の場合、同じトラックに留まること
    it("stays on same track when repeat is one", () => {
      const tracks = makeTracks(3);
      usePlayerStore.getState().setQueue(tracks, 1);
      usePlayerStore.setState({ repeat: "one" });
      usePlayerStore.getState().next();
      expect(usePlayerStore.getState().currentTrack?.id).toBe("t2");
    });

    // 空のキューで next() を呼んでもエラーにならないこと
    it("does nothing on empty queue", () => {
      usePlayerStore.getState().next();
      expect(usePlayerStore.getState().currentTrack).toBeNull();
    });
  });

  describe("previous", () => {
    // 再生開始3秒以内なら前のトラックに戻ること
    it("goes to previous track when within first 3 seconds", () => {
      const tracks = makeTracks(3);
      usePlayerStore.getState().setQueue(tracks, 1);
      usePlayerStore.setState({ currentTime: 1 });
      usePlayerStore.getState().previous();
      expect(usePlayerStore.getState().currentTrack?.id).toBe("t1");
    });

    // 再生開始3秒以降なら currentTime が 0 にリセットされること（トラック頭出し）
    it("restarts current track when past 3 seconds", () => {
      const tracks = makeTracks(3);
      usePlayerStore.getState().setQueue(tracks, 1);
      usePlayerStore.setState({ currentTime: 10 });
      usePlayerStore.getState().previous();
      expect(usePlayerStore.getState().currentTrack?.id).toBe("t2");
      expect(usePlayerStore.getState().currentTime).toBe(0);
    });

    // repeat=all で先頭トラック時に末尾に戻ること
    it("wraps to last track when repeat is all and at beginning", () => {
      const tracks = makeTracks(3);
      usePlayerStore.getState().setQueue(tracks, 0);
      usePlayerStore.setState({ repeat: "all", currentTime: 0 });
      usePlayerStore.getState().previous();
      expect(usePlayerStore.getState().currentTrack?.id).toBe("t3");
    });
  });

  describe("addToQueue / removeFromQueue / clearQueue", () => {
    // キューにトラックを追加できること
    it("adds tracks to the end of the queue", () => {
      usePlayerStore.getState().setQueue(makeTracks(1));
      usePlayerStore.getState().addToQueue(makeTracks(2));
      expect(usePlayerStore.getState().queue).toHaveLength(3);
    });

    // キューから指定インデックスのトラックを削除できること
    it("removes track at given index", () => {
      usePlayerStore.getState().setQueue(makeTracks(3));
      usePlayerStore.getState().removeFromQueue(1);
      expect(usePlayerStore.getState().queue).toHaveLength(2);
    });

    // 現在再生中より前のトラックを削除すると queueIndex が調整されること
    it("adjusts queueIndex when removing before current", () => {
      usePlayerStore.getState().setQueue(makeTracks(3), 2);
      usePlayerStore.getState().removeFromQueue(0);
      expect(usePlayerStore.getState().queueIndex).toBe(1);
    });

    // clearQueue でキューが空になり再生停止すること
    it("clears all queue state", () => {
      usePlayerStore.getState().setQueue(makeTracks(3));
      usePlayerStore.getState().play();
      usePlayerStore.getState().clearQueue();
      const state = usePlayerStore.getState();
      expect(state.queue).toHaveLength(0);
      expect(state.currentTrack).toBeNull();
      expect(state.isPlaying).toBe(false);
    });
  });

  describe("volume / mute", () => {
    // setVolume で音量が変更されること
    it("sets volume", () => {
      usePlayerStore.getState().setVolume(0.5);
      expect(usePlayerStore.getState().volume).toBe(0.5);
    });

    // 音量を0にすると自動的にミュートになること
    it("auto-mutes when volume set to 0", () => {
      usePlayerStore.getState().setVolume(0);
      expect(usePlayerStore.getState().muted).toBe(true);
    });

    // toggleMute でミュート状態がトグルされること
    it("toggles mute", () => {
      usePlayerStore.getState().toggleMute();
      expect(usePlayerStore.getState().muted).toBe(true);
      usePlayerStore.getState().toggleMute();
      expect(usePlayerStore.getState().muted).toBe(false);
    });
  });

  describe("shuffle / repeat", () => {
    // toggleShuffle でシャッフル状態がトグルされること
    it("toggles shuffle", () => {
      usePlayerStore.getState().toggleShuffle();
      expect(usePlayerStore.getState().shuffle).toBe(true);
    });

    // cycleRepeat で off → all → one → off の順に切り替わること
    it("cycles repeat: off → all → one → off", () => {
      const { cycleRepeat } = usePlayerStore.getState();
      cycleRepeat();
      expect(usePlayerStore.getState().repeat).toBe("all");
      usePlayerStore.getState().cycleRepeat();
      expect(usePlayerStore.getState().repeat).toBe("one");
      usePlayerStore.getState().cycleRepeat();
      expect(usePlayerStore.getState().repeat).toBe("off");
    });
  });
});
