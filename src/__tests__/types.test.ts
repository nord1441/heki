import { describe, it, expect } from "vitest";
import {
  formatDuration,
  formatDurationLong,
  MOOD_INFO,
  type MoodCategory,
} from "../types";

describe("formatDuration", () => {
  // 0秒のフォーマットが "0:00" になること
  it("formats zero seconds", () => {
    expect(formatDuration(0)).toBe("0:00");
  });

  // 60秒未満の場合、分が0でゼロ埋め秒数が表示されること
  it("formats seconds under one minute", () => {
    expect(formatDuration(5)).toBe("0:05");
    expect(formatDuration(59)).toBe("0:59");
  });

  // 分と秒が正しくフォーマットされること
  it("formats minutes and seconds", () => {
    expect(formatDuration(90)).toBe("1:30");
    expect(formatDuration(3661)).toBe("61:01");
  });

  // 小数の秒数は切り捨てられること
  it("truncates fractional seconds", () => {
    expect(formatDuration(125.7)).toBe("2:05");
  });
});

describe("formatDurationLong", () => {
  // 1時間未満の場合は "m:ss" 形式で表示されること
  it("formats durations under one hour without hours", () => {
    expect(formatDurationLong(0)).toBe("0:00");
    expect(formatDurationLong(90)).toBe("1:30");
  });

  // 1時間以上の場合は "h:mm:ss" 形式で表示されること
  it("formats durations of one hour or more with hours", () => {
    expect(formatDurationLong(3600)).toBe("1:00:00");
    expect(formatDurationLong(3661)).toBe("1:01:01");
    expect(formatDurationLong(7384)).toBe("2:03:04");
  });
});

describe("MOOD_INFO", () => {
  const expectedCategories: MoodCategory[] = [
    "energetic",
    "upbeat",
    "dance",
    "extreme",
    "emotional",
    "mellow",
    "relax",
    "lounge",
  ];

  // SensMe 由来の 8 種類すべてのムードカテゴリが定義されていること
  it("defines all 8 mood categories", () => {
    expect(Object.keys(MOOD_INFO)).toHaveLength(8);
    for (const cat of expectedCategories) {
      expect(MOOD_INFO[cat]).toBeDefined();
    }
  });

  // 各カテゴリの category フィールドがキーと一致すること
  it("each entry has matching category field", () => {
    for (const cat of expectedCategories) {
      expect(MOOD_INFO[cat].category).toBe(cat);
    }
  });

  // label, description, icon が空文字列でないこと
  it("each entry has non-empty label, description, and icon", () => {
    for (const cat of expectedCategories) {
      const info = MOOD_INFO[cat];
      expect(info.label.length).toBeGreaterThan(0);
      expect(info.description.length).toBeGreaterThan(0);
      expect(info.icon.length).toBeGreaterThan(0);
    }
  });
});
