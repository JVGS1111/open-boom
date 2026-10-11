import React, { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Search, Headphones, Check, X, RotateCcw, Sliders, Volume2 } from "lucide-react";

export interface AutoEqProfileSummary {
  id: string;
  brand: string;
  model: string;
  name: string;
  target_curve: string;
  preamp_gain_db: number;
  filter_count: number;
  score: number;
}

export interface AutoEqPopoverProps {
  isOpen: boolean;
  onClose: () => void;
  activeProfileId?: string | null;
  onSelectProfile: (profile: AutoEqProfileSummary) => void;
  onClearProfile: () => void;
  onSearch?: (query: string) => Promise<AutoEqProfileSummary[]>;
}

// Fallback top popular models when running in browser or offline mock mode
const DEFAULT_FALLBACK_RECOMMENDATIONS: AutoEqProfileSummary[] = [
  {
    id: "hyperx-cloud-ii-oratory1990",
    brand: "HyperX",
    model: "Cloud II",
    name: "HyperX Cloud II",
    target_curve: "oratory1990 - Harman Over-Ear 2018",
    preamp_gain_db: -4.8,
    filter_count: 7,
    score: 100,
  },
  {
    id: "sennheiser-hd-600-oratory1990",
    brand: "Sennheiser",
    model: "HD 600",
    name: "Sennheiser HD 600",
    target_curve: "oratory1990 - Harman Over-Ear 2018",
    preamp_gain_db: -5.2,
    filter_count: 8,
    score: 100,
  },
  {
    id: "sony-wh-1000xm5-oratory1990",
    brand: "Sony",
    model: "WH-1000XM5",
    name: "Sony WH-1000XM5",
    target_curve: "oratory1990 - Harman Over-Ear 2018",
    preamp_gain_db: -4.2,
    filter_count: 8,
    score: 100,
  },
  {
    id: "apple-airpods-max-oratory1990",
    brand: "Apple",
    model: "AirPods Max",
    name: "Apple AirPods Max",
    target_curve: "oratory1990 - Harman Over-Ear 2018",
    preamp_gain_db: -3.8,
    filter_count: 7,
    score: 100,
  },
  {
    id: "beyerdynamic-dt-770-pro-80-ohm-oratory1990",
    brand: "Beyerdynamic",
    model: "DT 770 Pro 80 Ohm",
    name: "Beyerdynamic DT 770 Pro 80 Ohm",
    target_curve: "oratory1990 - Harman Over-Ear 2018",
    preamp_gain_db: -4.5,
    filter_count: 8,
    score: 100,
  },
  {
    id: "audio-technica-ath-m50x-oratory1990",
    brand: "Audio-Technica",
    model: "ATH-M50x",
    name: "Audio-Technica ATH-M50x",
    target_curve: "oratory1990 - Harman Over-Ear 2018",
    preamp_gain_db: -4.0,
    filter_count: 7,
    score: 100,
  },
  {
    id: "apple-airpods-pro-2-crinacle",
    brand: "Apple",
    model: "AirPods Pro 2",
    name: "Apple AirPods Pro 2",
    target_curve: "crinacle - Harman In-Ear 2019v2",
    preamp_gain_db: -3.5,
    filter_count: 6,
    score: 100,
  },
  {
    id: "sennheiser-hd-560s-oratory1990",
    brand: "Sennheiser",
    model: "HD 560S",
    name: "Sennheiser HD 560S",
    target_curve: "oratory1990 - Harman Over-Ear 2018",
    preamp_gain_db: -3.9,
    filter_count: 7,
    score: 100,
  },
];

export const AutoEqPopover: React.FC<AutoEqPopoverProps> = ({
  isOpen,
  onClose,
  activeProfileId,
  onSelectProfile,
  onClearProfile,
  onSearch,
}) => {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<AutoEqProfileSummary[]>(
    DEFAULT_FALLBACK_RECOMMENDATIONS
  );
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isLoading, setIsLoading] = useState(false);

  const popoverRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const searchSeqRef = useRef<number>(0);
  const isKeyboardNavRef = useRef<boolean>(false);

  // Execute search query against Tauri IPC or custom handler with sub-50ms latency
  const executeSearch = useCallback(
    async (searchQuery: string) => {
      const seq = ++searchSeqRef.current;
      setIsLoading(true);
      try {
        let items: AutoEqProfileSummary[] = [];
        if (onSearch) {
          items = await onSearch(searchQuery);
        } else {
          try {
            items = await invoke<AutoEqProfileSummary[]>("search_autoeq_profiles", {
              query: searchQuery,
            });
          } catch {
            // Fallback for browser preview or tests
            const q = searchQuery.toLowerCase().trim();
            if (!q) {
              items = DEFAULT_FALLBACK_RECOMMENDATIONS;
            } else {
              items = DEFAULT_FALLBACK_RECOMMENDATIONS.filter(
                (p) =>
                  p.name.toLowerCase().includes(q) ||
                  p.model.toLowerCase().includes(q) ||
                  p.brand.toLowerCase().includes(q) ||
                  p.target_curve.toLowerCase().includes(q)
              );
            }
          }
        }
        if (seq === searchSeqRef.current) {
          setResults(items);
          setSelectedIndex(0);
        }
      } finally {
        if (seq === searchSeqRef.current) {
          setIsLoading(false);
        }
      }
    },
    [onSearch]
  );

  // Fetch initial popular recommendations and auto-focus search input upon opening
  useEffect(() => {
    if (isOpen) {
      setQuery("");
      executeSearch("");
      setTimeout(() => {
        inputRef.current?.focus();
      }, 50);
    }
  }, [isOpen, executeSearch]);

  // Debounced search on query change
  useEffect(() => {
    if (!isOpen) return;
    const timer = setTimeout(() => {
      executeSearch(query);
    }, 20);
    return () => clearTimeout(timer);
  }, [query, isOpen, executeSearch]);

  // Close on Escape or click outside
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        onClose();
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        isKeyboardNavRef.current = true;
        setSelectedIndex((prev) =>
          results.length > 0 ? (prev + 1) % results.length : 0
        );
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        isKeyboardNavRef.current = true;
        setSelectedIndex((prev) =>
          results.length > 0
            ? (prev - 1 + results.length) % results.length
            : 0
        );
      } else if (e.key === "Enter") {
        if (results.length > 0 && selectedIndex >= 0 && selectedIndex < results.length) {
          e.preventDefault();
          onSelectProfile(results[selectedIndex]);
          onClose();
        }
      }
    };

    const handleClickOutside = (e: MouseEvent) => {
      if (popoverRef.current && !popoverRef.current.contains(e.target as Node)) {
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isOpen, results, selectedIndex, onClose, onSelectProfile]);

  // Keep selected item scrolled into view
  useEffect(() => {
    if (listRef.current) {
      const selectedEl = listRef.current.children[selectedIndex] as HTMLElement;
      if (selectedEl) {
        selectedEl.scrollIntoView({ block: "nearest" });
      }
    }
  }, [selectedIndex]);

  if (!isOpen) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="AutoEq Headphone Profile Search"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-in fade-in duration-150"
    >
      <div
        ref={popoverRef}
        className="w-full max-w-lg bg-surface-card border border-border-default rounded-xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh] animate-in zoom-in-95 duration-150 text-text-primary"
      >
        {/* 1. Header with Search Input */}
        <div className="p-3.5 border-b border-border-subtle bg-surface-base/80 flex flex-col gap-2.5">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="p-1.5 rounded-lg bg-accent-blue/15 text-accent-cyan border border-accent-blue/30">
                <Headphones className="w-4 h-4" />
              </div>
              <div>
                <h3 className="text-xs font-bold tracking-wide uppercase text-text-primary">
                  AutoEq Headphone Profiles
                </h3>
                <p className="text-[10px] text-text-muted">
                  4,000+ calibrated models &bull; Zero network telemetry &bull; Offline
                </p>
              </div>
            </div>

            <div className="flex items-center gap-1.5">
              {activeProfileId && (
                <button
                  type="button"
                  onClick={() => {
                    onClearProfile();
                    onClose();
                  }}
                  className="flex items-center gap-1 px-2 py-1 rounded text-[10px] font-medium bg-surface-card border border-border-subtle hover:border-red-500/40 hover:text-red-400 text-text-muted transition-colors cursor-pointer"
                  title="Clear active calibration profile"
                >
                  <RotateCcw className="w-3 h-3" />
                  <span>Reset to Flat</span>
                </button>
              )}
              <button
                type="button"
                onClick={onClose}
                aria-label="Close AutoEq Popover"
                className="p-1 rounded-md text-text-muted hover:text-text-primary hover:bg-surface-card border border-transparent hover:border-border-subtle transition-all cursor-pointer"
              >
                <X className="w-4 h-4" />
              </button>
            </div>
          </div>

          {/* Search Input Field */}
          <div className="relative flex items-center">
            <Search className="w-4 h-4 text-accent-cyan absolute left-3 pointer-events-none" />
            <input
              ref={inputRef}
              type="text"
              role="combobox"
              aria-expanded={isOpen}
              aria-autocomplete="list"
              aria-controls="autoeq-listbox"
              aria-activedescendant={
                results.length > 0 && results[selectedIndex]
                  ? `autoeq-opt-${results[selectedIndex].id}`
                  : undefined
              }
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search by brand or model (e.g. HyperX Cloud II, HD 600, XM5)..."
              aria-label="Search headphone models"
              className="w-full pl-9 pr-8 py-2 bg-surface-base border border-border-subtle focus:border-accent-cyan rounded-lg text-xs text-text-primary placeholder:text-text-muted outline-none transition-all shadow-inner"
            />
            {query && (
              <button
                type="button"
                onClick={() => setQuery("")}
                className="absolute right-2.5 p-0.5 rounded text-text-muted hover:text-text-primary"
                title="Clear query"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            )}
          </div>
        </div>

        {/* 2. Results List View */}
        <div
          ref={listRef}
          id="autoeq-listbox"
          role="listbox"
          aria-label="Headphone Profile Results"
          className="flex-1 overflow-y-auto p-2 space-y-1.5 min-h-[160px] max-h-[380px]"
        >
          {results.length > 0 ? (
            results.map((profile, idx) => {
              const isSelected = idx === selectedIndex;
              const isActive = profile.id === activeProfileId;

              return (
                <div
                  key={profile.id}
                  id={`autoeq-opt-${profile.id}`}
                  role="option"
                  aria-selected={isSelected}
                  onClick={() => {
                    onSelectProfile(profile);
                    onClose();
                  }}
                  onMouseEnter={() => {
                    if (!isKeyboardNavRef.current) {
                      setSelectedIndex(idx);
                    }
                  }}
                  onPointerMove={() => {
                    isKeyboardNavRef.current = false;
                  }}
                  className={`p-2.5 rounded-lg border transition-all cursor-pointer flex items-center justify-between select-none ${
                    isSelected
                      ? "bg-accent-blue/15 border-accent-cyan/80 shadow-md"
                      : "bg-surface-base/60 border-border-subtle hover:bg-surface-base hover:border-border-default"
                  }`}
                >
                  <div className="flex-1 pr-3">
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-bold text-text-primary">
                        <span className="text-accent-cyan font-semibold mr-1">
                          {profile.brand}
                        </span>
                        {profile.model}
                      </span>
                      {isActive && (
                        <span className="flex items-center gap-1 text-[9px] font-semibold uppercase bg-accent-blue/30 text-accent-cyan px-1.5 py-0.5 rounded border border-accent-blue/40">
                          <Check className="w-2.5 h-2.5" />
                          Active
                        </span>
                      )}
                    </div>

                    <div className="flex items-center gap-2 mt-1 text-[10px] text-text-muted">
                      <span className="truncate max-w-[220px]" title={profile.target_curve}>
                        {profile.target_curve}
                      </span>
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5 flex-shrink-0 text-[10px] font-mono">
                    <span
                      className="px-1.5 py-0.5 rounded bg-surface-card border border-border-subtle text-text-secondary flex items-center gap-1"
                      title="Number of calibrated parametric biquad filters"
                    >
                      <Sliders className="w-2.5 h-2.5 text-accent-blue" />
                      {profile.filter_count} bands
                    </span>
                    <span
                      className="px-1.5 py-0.5 rounded bg-surface-card border border-border-subtle text-accent-cyan flex items-center gap-1"
                      title="Preamp attenuation offset to prevent digital clipping"
                    >
                      <Volume2 className="w-2.5 h-2.5" />
                      {profile.preamp_gain_db.toFixed(1)} dB
                    </span>
                  </div>
                </div>
              );
            })
          ) : (
            <div className="py-12 flex flex-col items-center justify-center text-center px-4">
              <Headphones className="w-8 h-8 text-text-muted/40 mb-2" />
              <p className="text-xs font-medium text-text-secondary">
                No headphones found matching &ldquo;{query}&rdquo;
              </p>
              <p className="text-[10px] text-text-muted mt-0.5">
                Try searching by brand (HyperX, Sony, Sennheiser) or model number
              </p>
            </div>
          )}
        </div>

        {/* 3. Footer Navigation Tips */}
        <div className="px-3.5 py-2 border-t border-border-subtle bg-surface-base/80 flex items-center justify-between text-[10px] text-text-muted font-mono select-none">
          <div className="flex items-center gap-2">
            <span>&uarr;&darr; Navigate</span>
            <span>&bull;</span>
            <span>Enter Select</span>
            <span>&bull;</span>
            <span>Esc Dismiss</span>
          </div>
          <div>
            {isLoading ? (
              <span className="text-accent-cyan animate-pulse">Searching...</span>
            ) : (
              <span>
                {results.length} result{results.length === 1 ? "" : "s"}
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default AutoEqPopover;
