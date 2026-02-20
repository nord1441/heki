import "@testing-library/jest-dom/vitest";

// Tauri ランタイムのモック（テスト環境では window.__TAURI_INTERNALS__ が存在しないため）
Object.defineProperty(window, "__TAURI_INTERNALS__", {
  value: {
    invoke: async () => {},
    convertFileSrc: (path: string, protocol = "asset") =>
      `${protocol}://localhost/${encodeURIComponent(path)}`,
    transformCallback: () => 0,
  },
  writable: true,
});
