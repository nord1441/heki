import { useRef, useState, useEffect } from "react";
import { useLibraryStore } from "../stores/libraryStore";

export function SearchBar() {
  const { searchQuery, setSearchQuery } = useLibraryStore();
  const [focused, setFocused] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "f") {
        e.preventDefault();
        inputRef.current?.focus();
      }
      if (e.key === "Escape" && focused) {
        setSearchQuery("");
        inputRef.current?.blur();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [focused]);

  return (
    <div className={`search-bar ${focused ? "focused" : ""}`}>
      <span className="search-icon">/</span>
      <input
        ref={inputRef}
        type="text"
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.target.value)}
        onFocus={() => setFocused(true)}
        onBlur={() => setFocused(false)}
        placeholder="SEARCH..."
        className="search-input"
        spellCheck={false}
      />
      {searchQuery && (
        <button
          className="search-clear"
          onClick={() => {
            setSearchQuery("");
            inputRef.current?.focus();
          }}
        >
          x
        </button>
      )}
    </div>
  );
}
