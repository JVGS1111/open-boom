import React, { useState, useEffect, useRef } from "react";
import {
  SlidersHorizontal,
  Check,
  Plus,
  Pencil,
  Trash2,
  X,
  Sparkles,
  Bookmark,
} from "lucide-react";

export interface EqPreset {
  id: string;
  name: string;
  category: string;
  is_factory: boolean;
  gains_db: number[];
}

/**
 * Built-in 15 calibrated 31-band ISO 1/3-octave factory presets (FR11).
 * Matches `PresetStore::factory_presets()` in `crates/open-boom-storage`.
 */
export const FACTORY_EQ_PRESETS: EqPreset[] = [
  {
    id: "flat",
    name: "Flat",
    category: "Studio",
    is_factory: true,
    gains_db: new Array(31).fill(0.0),
  },
  {
    id: "bass-boost",
    name: "Bass Boost",
    category: "Signature",
    is_factory: true,
    gains_db: [
      6.0, 6.0, 5.5, 5.0, 4.5, 4.0, 3.5, 2.5, 1.8, 1.0, 0.5, 0.0, 0.0, 0.0, 0.0,
      0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
      0.0,
    ],
  },
  {
    id: "treble-boost",
    name: "Treble Boost",
    category: "Signature",
    is_factory: true,
    gains_db: [
      0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
      0.0, 0.0, 0.0, 0.5, 1.0, 1.5, 2.2, 3.0, 3.8, 4.5, 5.0, 5.5, 5.8, 6.0, 5.5,
      4.5,
    ],
  },
  {
    id: "acoustic",
    name: "Acoustic",
    category: "Music",
    is_factory: true,
    gains_db: [
      3.5, 3.5, 3.2, 2.8, 2.2, 1.5, 1.0, 0.8, 0.5, 0.2, 0.0, -0.5, -0.5, 0.0, 0.5,
      1.0, 1.5, 2.0, 2.2, 2.5, 2.5, 2.0, 1.8, 2.0, 2.5, 3.0, 3.2, 3.0, 2.5, 2.0,
      1.5,
    ],
  },
  {
    id: "classical",
    name: "Classical",
    category: "Music",
    is_factory: true,
    gains_db: [
      3.5, 3.2, 2.8, 2.4, 1.8, 1.2, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.3, -0.5,
      -0.5, -0.3, 0.0, 0.3, 0.8, 1.2, 1.6, 2.0, 2.4, 2.8, 3.0, 3.2, 3.2, 3.0, 2.5,
      2.0,
    ],
  },
  {
    id: "electronic",
    name: "Electronic",
    category: "Music",
    is_factory: true,
    gains_db: [
      5.0, 5.5, 5.2, 4.5, 3.8, 2.8, 1.5, 0.5, -0.5, -1.2, -1.5, -1.2, -0.8, -0.5,
      0.0, 0.5, 0.8, 1.0, 0.8, 0.5, 1.0, 1.8, 2.5, 3.2, 3.8, 4.2, 4.5, 4.5, 4.0,
      3.5, 2.5,
    ],
  },
  {
    id: "gaming",
    name: "Gaming",
    category: "Media",
    is_factory: true,
    gains_db: [
      2.0, 2.5, 3.0, 2.8, 2.2, 1.5, 0.5, -0.5, -1.2, -1.8, -2.0, -1.5, -0.8, 0.0,
      0.8, 1.5, 2.2, 2.8, 3.5, 4.2, 4.5, 4.2, 3.8, 3.2, 2.5, 2.0, 1.8, 1.5, 1.0,
      0.5, 0.0,
    ],
  },
  {
    id: "hip-hop",
    name: "Hip-Hop",
    category: "Music",
    is_factory: true,
    gains_db: [
      5.5, 6.0, 5.8, 5.2, 4.5, 3.8, 2.8, 1.8, 0.8, 0.0, -0.8, -1.2, -1.5, -1.2,
      -0.8, -0.3, 0.2, 0.8, 1.2, 1.5, 1.5, 1.2, 1.0, 1.2, 1.8, 2.2, 2.8, 3.0, 2.8,
      2.2, 1.5,
    ],
  },
  {
    id: "jazz",
    name: "Jazz",
    category: "Music",
    is_factory: true,
    gains_db: [
      3.2, 3.5, 3.2, 2.8, 2.4, 2.0, 1.5, 1.0, 0.5, 0.2, 0.0, -0.5, -0.8, -0.8,
      -0.5, 0.0, 0.5, 1.0, 1.4, 1.8, 2.0, 2.0, 1.8, 1.8, 2.0, 2.4, 2.8, 2.8, 2.5,
      2.0, 1.5,
    ],
  },
  {
    id: "movie",
    name: "Movie",
    category: "Media",
    is_factory: true,
    gains_db: [
      5.0, 5.5, 5.0, 4.2, 3.5, 2.5, 1.5, 0.5, -0.5, -1.0, -1.2, -1.0, -0.5, 0.2,
      1.0, 1.8, 2.5, 3.0, 3.2, 3.0, 2.5, 2.0, 1.8, 2.0, 2.5, 3.0, 3.2, 3.0, 2.5,
      2.0, 1.5,
    ],
  },
  {
    id: "pop",
    name: "Pop",
    category: "Music",
    is_factory: true,
    gains_db: [
      2.0, 2.5, 2.8, 3.0, 2.8, 2.2, 1.5, 0.8, 0.2, -0.2, -0.5, -0.5, 0.0, 0.8,
      1.5, 2.2, 2.8, 3.0, 2.8, 2.5, 2.0, 1.8, 2.0, 2.2, 2.5, 2.8, 3.0, 2.8, 2.2,
      1.8, 1.2,
    ],
  },
  {
    id: "rock",
    name: "Rock",
    category: "Music",
    is_factory: true,
    gains_db: [
      4.0, 4.5, 4.5, 4.2, 3.8, 3.2, 2.5, 1.5, 0.5, -0.5, -1.2, -1.8, -1.8, -1.2,
      -0.5, 0.5, 1.5, 2.2, 2.8, 3.2, 3.5, 3.8, 4.0, 4.0, 3.8, 3.5, 3.5, 3.2, 2.8,
      2.2, 1.5,
    ],
  },
  {
    id: "vocal",
    name: "Vocal",
    category: "Studio",
    is_factory: true,
    gains_db: [
      -3.0, -2.8, -2.5, -2.0, -1.5, -1.0, -0.5, 0.0, 0.5, 1.2, 1.8, 2.5, 3.0, 3.5,
      3.8, 4.0, 4.2, 4.2, 4.0, 3.8, 3.5, 3.0, 2.5, 1.8, 1.0, 0.5, 0.0, -0.5, -1.0,
      -1.5, -2.0,
    ],
  },
  {
    id: "lounge",
    name: "Lounge",
    category: "Music",
    is_factory: true,
    gains_db: [
      3.0, 3.5, 3.8, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0, 0.5, 0.2, 0.0, -0.3, -0.8,
      -1.2, -1.5, -1.5, -1.2, -0.8, -0.5, 0.0, 0.5, 1.0, 1.5, 1.8, 2.0, 2.2, 2.0,
      1.8, 1.5, 1.0,
    ],
  },
  {
    id: "deep",
    name: "Deep",
    category: "Signature",
    is_factory: true,
    gains_db: [
      5.0, 5.5, 5.5, 5.0, 4.5, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0, 0.5, 0.0, -0.5,
      -1.0, -1.2, -1.5, -1.5, -1.2, -1.0, -0.8, -0.5, -0.5, -0.8, -1.0, -1.2,
      -1.5, -1.8, -2.0, -2.5,
    ],
  },
];

/**
 * Sanitizes a preset name to its canonical slug representation (matching Rust `PresetStore::sanitize_preset_slug`).
 */
export function toPresetSlug(input: string): string {
  return input
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 64)
    .replace(/-+$/g, "");
}

/**
 * Determines whether the current 31-band equalizer gains deviate from the active preset's curve
 * by more than `toleranceDb` (default 0.05 dB) (EXPERIENCE.md §5.2).
 */
export function isPresetDirty(
  currentGains: number[],
  presetGains: number[],
  toleranceDb: number = 0.05
): boolean {
  if (currentGains.length !== presetGains.length) {
    return true;
  }
  for (let i = 0; i < currentGains.length; i++) {
    const a = currentGains[i] ?? 0.0;
    const b = presetGains[i] ?? 0.0;
    if (Math.abs(a - b) > toleranceDb) {
      return true;
    }
  }
  return false;
}

/**
 * Formats the active preset name with a modified asterisk indicator (`Rock *`) when dirty (EXPERIENCE.md §5.2).
 */
export function formatPresetDisplayLabel(
  presetName: string,
  isDirty: boolean
): string {
  const base = presetName.trim() || "Flat";
  return isDirty ? `${base} *` : base;
}

/**
 * Validates a user-provided custom preset name against empty names, reserved names, and factory collisions.
 */
export function validateCustomPresetName(
  name: string,
  existingPresets: EqPreset[],
  editingPresetId?: string
): { valid: boolean; error?: string } {
  const trimmed = name.trim();
  if (!trimmed) {
    return { valid: false, error: "Preset name cannot be empty." };
  }

  const slug = toPresetSlug(trimmed);
  if (!slug) {
    return {
      valid: false,
      error: "Preset name must contain at least one letter or number.",
    };
  }

  const reserved = new Set([
    "con",
    "prn",
    "aux",
    "nul",
    "com1",
    "com2",
    "com3",
    "com4",
    "com5",
    "com6",
    "com7",
    "com8",
    "com9",
    "lpt1",
    "lpt2",
    "lpt3",
    "lpt4",
    "lpt5",
    "lpt6",
    "lpt7",
    "lpt8",
    "lpt9",
  ]);
  if (reserved.has(slug)) {
    return {
      valid: false,
      error: `'${trimmed}' is a reserved system name.`,
    };
  }

  const factoryCollision = FACTORY_EQ_PRESETS.some(
    (fp) =>
      fp.name.toLowerCase() === trimmed.toLowerCase() || fp.id === slug
  );
  if (factoryCollision) {
    return {
      valid: false,
      error: `'${trimmed}' is a protected factory preset name.`,
    };
  }

  if (editingPresetId) {
    const customCollision = existingPresets.some(
      (p) =>
        !p.is_factory &&
        p.id !== editingPresetId &&
        (p.name.toLowerCase() === trimmed.toLowerCase() || p.id === slug)
    );
    if (customCollision) {
      return {
        valid: false,
        error: `A custom preset named '${trimmed}' already exists.`,
      };
    }
  }

  return { valid: true };
}

export interface PresetSelectorPopoverProps {
  isOpen: boolean;
  onClose: () => void;
  presets: EqPreset[];
  activePresetName: string;
  currentGains: number[];
  isDirty: boolean;
  onSelectPreset: (preset: EqPreset) => void;
  onSaveCustomPreset: (name: string) => Promise<void> | void;
  onRenameCustomPreset: (oldId: string, newName: string) => Promise<void> | void;
  onDeleteCustomPreset: (preset: EqPreset) => Promise<void> | void;
}

export const PresetSelectorPopover: React.FC<PresetSelectorPopoverProps> = ({
  isOpen,
  onClose,
  presets,
  activePresetName,
  isDirty,
  onSelectPreset,
  onSaveCustomPreset,
  onRenameCustomPreset,
  onDeleteCustomPreset,
}) => {
  const [isCreating, setIsCreating] = useState(false);
  const [newPresetName, setNewPresetName] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState("");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const createInputRef = useRef<HTMLInputElement>(null);

  const effectivePresets =
    presets && presets.length >= 15 ? presets : FACTORY_EQ_PRESETS;
  const factoryPresets = effectivePresets.filter((p) => p.is_factory);
  const customPresets = effectivePresets.filter((p) => !p.is_factory);

  useEffect(() => {
    if (isOpen) {
      setErrorMsg(null);
      setEditingId(null);
      setIsCreating(false);
      setNewPresetName("");
    }
  }, [isOpen]);

  useEffect(() => {
    if (isCreating) {
      setTimeout(() => createInputRef.current?.focus(), 10);
    }
  }, [isCreating]);

  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        e.stopImmediatePropagation();
        if (isCreating) {
          setIsCreating(false);
          setErrorMsg(null);
        } else if (editingId) {
          setEditingId(null);
          setErrorMsg(null);
        } else {
          onClose();
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown, true);
    return () => window.removeEventListener("keydown", handleKeyDown, true);
  }, [isOpen, isCreating, editingId, onClose]);

  const handleStartCreate = () => {
    setEditingId(null);
    setErrorMsg(null);
    setNewPresetName(
      isDirty && activePresetName ? `${activePresetName} Custom` : ""
    );
    setIsCreating(true);
  };

  const handleConfirmCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    const validation = validateCustomPresetName(
      newPresetName,
      effectivePresets
    );
    if (!validation.valid) {
      setErrorMsg(validation.error || "Invalid preset name.");
      return;
    }
    try {
      await onSaveCustomPreset(newPresetName.trim());
      setIsCreating(false);
      setNewPresetName("");
      setErrorMsg(null);
    } catch (err) {
      setErrorMsg(String(err));
    }
  };

  const handleStartRename = (preset: EqPreset, e: React.MouseEvent) => {
    e.stopPropagation();
    setIsCreating(false);
    setErrorMsg(null);
    setEditingId(preset.id);
    setEditingName(preset.name);
  };

  const handleConfirmRename = async (e: React.FormEvent, oldId: string) => {
    e.preventDefault();
    e.stopPropagation();
    const validation = validateCustomPresetName(
      editingName,
      effectivePresets,
      oldId
    );
    if (!validation.valid) {
      setErrorMsg(validation.error || "Invalid preset name.");
      return;
    }
    try {
      await onRenameCustomPreset(oldId, editingName.trim());
      setEditingId(null);
      setEditingName("");
      setErrorMsg(null);
    } catch (err) {
      setErrorMsg(String(err));
    }
  };

  const handleDelete = async (preset: EqPreset, e: React.MouseEvent) => {
    e.stopPropagation();
    setErrorMsg(null);
    try {
      await onDeleteCustomPreset(preset);
    } catch (err) {
      setErrorMsg(String(err));
    }
  };

  const displayLabel = formatPresetDisplayLabel(activePresetName, isDirty);

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-hidden={!isOpen}
      aria-label="Equalizer Presets"
      style={!isOpen ? { display: "none" } : undefined}
      onClick={onClose}
      className={`fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 transition-opacity duration-150 ${
        isOpen
          ? "opacity-100 pointer-events-auto"
          : "opacity-0 pointer-events-none"
      }`}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-[500px] bg-surface-overlay border border-border-default rounded-xl shadow-2xl overflow-hidden flex flex-col max-h-[460px]"
      >
        {/* 1. Header */}
        <div className="px-4 py-3 border-b border-border-subtle bg-surface-card/80 flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className="p-1.5 rounded-lg bg-accent-blue/15 text-accent-cyan border border-accent-blue/30">
              <SlidersHorizontal className="w-4 h-4" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h2 className="text-xs font-semibold tracking-wider text-text-primary uppercase">
                  Equalizer Presets
                </h2>
                <span className="text-[10px] font-mono text-accent-cyan bg-surface-base px-2 py-0.5 rounded border border-border-subtle">
                  {displayLabel}
                </span>
                {isDirty && (
                  <span className="text-[9px] font-mono text-amber-400 bg-amber-500/10 px-1.5 py-0.5 rounded border border-amber-500/30">
                    Modified
                  </span>
                )}
              </div>
              <p className="text-[10px] text-text-muted">
                {factoryPresets.length} Factory Genre Curves •{" "}
                {customPresets.length} Custom Saved
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={handleStartCreate}
              className={`flex items-center gap-1 px-2.5 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
                isDirty
                  ? "bg-accent-cyan text-background font-semibold shadow-cyan-glow-sm hover:bg-accent-cyan/90"
                  : "bg-accent-blue/15 text-accent-cyan hover:bg-accent-blue/25 border border-accent-blue/30"
              }`}
              title="Save current 31-band curve as a custom preset"
            >
              <Plus className="w-3.5 h-3.5" />
              <span>Save As Custom Preset</span>
            </button>

            <button
              type="button"
              onClick={onClose}
              aria-label="Close Preset Selector"
              className="p-1 rounded-md text-text-muted hover:text-text-primary hover:bg-surface-base transition-colors cursor-pointer"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* 2. Inline Save As Custom Preset Form */}
        {isCreating && (
          <form
            onSubmit={handleConfirmCreate}
            className="px-4 py-2.5 bg-surface-base border-b border-border-subtle flex flex-col gap-1.5"
          >
            <div className="flex items-center gap-2">
              <input
                ref={createInputRef}
                type="text"
                value={newPresetName}
                onChange={(e) => {
                  setNewPresetName(e.target.value);
                  setErrorMsg(null);
                }}
                placeholder="Enter custom preset name (e.g. My Late Night EQ)..."
                maxLength={48}
                className="flex-1 bg-surface-card border border-border-default focus:border-accent-cyan rounded-md px-2.5 py-1 text-xs text-text-primary placeholder:text-text-muted focus:outline-none"
              />
              <button
                type="submit"
                className="px-3 py-1 rounded-md text-xs font-semibold bg-accent-cyan text-background hover:bg-accent-cyan/90 cursor-pointer"
              >
                Save
              </button>
              <button
                type="button"
                onClick={() => {
                  setIsCreating(false);
                  setErrorMsg(null);
                }}
                className="px-2.5 py-1 rounded-md text-xs text-text-secondary hover:text-text-primary bg-surface-card border border-border-subtle cursor-pointer"
              >
                Cancel
              </button>
            </div>
            {errorMsg && (
              <span className="text-[10px] text-rose-400 font-medium">
                {errorMsg}
              </span>
            )}
          </form>
        )}

        {!isCreating && errorMsg && (
          <div className="px-4 py-1.5 bg-rose-500/10 border-b border-rose-500/30 text-[10px] text-rose-400">
            {errorMsg}
          </div>
        )}

        {/* 3. Scrollable Presets List */}
        <div className="flex-1 overflow-y-auto p-3 space-y-3">
          {/* Custom Presets Section */}
          <div>
            <div className="flex items-center justify-between px-1 mb-1.5">
              <span className="text-[10px] font-semibold uppercase tracking-wider text-text-muted flex items-center gap-1.5">
                <Bookmark className="w-3 h-3 text-accent-cyan" />
                User Custom Presets ({customPresets.length})
              </span>
              <span className="text-[9px] font-mono text-text-muted">
                %APPDATA%\OpenBoom\presets\
              </span>
            </div>

            {customPresets.length === 0 ? (
              <div className="px-3 py-2 rounded-lg bg-surface-base/60 border border-border-subtle/60 text-[11px] text-text-muted flex items-center justify-between">
                <span>
                  Adjust any fader and click &ldquo;Save As Custom Preset&rdquo; to store your personal curve.
                </span>
              </div>
            ) : (
              <div className="grid grid-cols-1 gap-1">
                {customPresets.map((preset) => {
                  const isSelected =
                    preset.name.toLowerCase() === activePresetName.toLowerCase();
                  const isEditingThis = editingId === preset.id;

                  if (isEditingThis) {
                    return (
                      <form
                        key={preset.id}
                        onSubmit={(e) => handleConfirmRename(e, preset.id)}
                        className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-surface-base border border-accent-cyan"
                      >
                        <input
                          type="text"
                          value={editingName}
                          onChange={(e) => {
                            setEditingName(e.target.value);
                            setErrorMsg(null);
                          }}
                          maxLength={48}
                          autoFocus
                          className="flex-1 bg-surface-card border border-border-default rounded px-2 py-0.5 text-xs text-text-primary focus:outline-none focus:border-accent-cyan"
                        />
                        <button
                          type="submit"
                          className="px-2.5 py-0.5 rounded text-[11px] font-semibold bg-accent-cyan text-background cursor-pointer"
                        >
                          Rename
                        </button>
                        <button
                          type="button"
                          onClick={() => {
                            setEditingId(null);
                            setErrorMsg(null);
                          }}
                          className="px-2 py-0.5 rounded text-[11px] text-text-muted hover:text-text-primary cursor-pointer"
                        >
                          Cancel
                        </button>
                      </form>
                    );
                  }

                  return (
                    <div
                      key={preset.id}
                      onClick={() => {
                        onSelectPreset(preset);
                        onClose();
                      }}
                      className={`flex items-center justify-between px-3 py-1.5 rounded-lg border transition-all cursor-pointer group ${
                        isSelected
                          ? "bg-accent-blue/15 border-accent-cyan text-text-primary"
                          : "bg-surface-base/70 hover:bg-surface-card border-border-subtle hover:border-border-default text-text-secondary hover:text-text-primary"
                      }`}
                    >
                      <div className="flex items-center gap-2">
                        {isSelected ? (
                          <Check className="w-3.5 h-3.5 text-accent-cyan shrink-0" />
                        ) : (
                          <span className="w-3.5 h-3.5 shrink-0" />
                        )}
                        <span className="text-xs font-medium">
                          {isSelected && isDirty
                            ? `${preset.name} *`
                            : preset.name}
                        </span>
                        <span className="text-[9px] font-mono px-1.5 py-0.2 rounded bg-surface-card text-accent-cyan border border-border-subtle">
                          Custom
                        </span>
                      </div>

                      <div className="flex items-center gap-1">
                        <button
                          type="button"
                          onClick={(e) => handleStartRename(preset, e)}
                          aria-label={`Rename ${preset.name}`}
                          title={`Rename ${preset.name}`}
                          className="p-1 rounded hover:bg-surface-overlay text-text-muted hover:text-accent-cyan transition-colors cursor-pointer"
                        >
                          <Pencil className="w-3 h-3" />
                        </button>
                        <button
                          type="button"
                          onClick={(e) => handleDelete(preset, e)}
                          aria-label={`Delete ${preset.name}`}
                          title={`Delete ${preset.name}`}
                          className="p-1 rounded hover:bg-rose-500/15 text-text-muted hover:text-rose-400 transition-colors cursor-pointer"
                        >
                          <Trash2 className="w-3 h-3" />
                        </button>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>

          {/* Factory Presets Section (15 Genre Presets - FR11) */}
          <div>
            <div className="flex items-center justify-between px-1 mb-1.5">
              <span className="text-[10px] font-semibold uppercase tracking-wider text-text-muted flex items-center gap-1.5">
                <Sparkles className="w-3 h-3 text-accent-blue" />
                Factory Genre Presets ({factoryPresets.length})
              </span>
              <span className="text-[9px] font-mono text-text-muted">
                31-Band ISO Calibrated
              </span>
            </div>

            <div className="grid grid-cols-3 gap-1.5">
              {factoryPresets.map((preset) => {
                const isSelected =
                  preset.name.toLowerCase() === activePresetName.toLowerCase();
                return (
                  <button
                    key={preset.id}
                    type="button"
                    onClick={() => {
                      onSelectPreset(preset);
                      onClose();
                    }}
                    className={`flex items-center justify-between px-2.5 py-2 rounded-lg border text-left transition-all cursor-pointer ${
                      isSelected
                        ? "bg-accent-blue/20 border-accent-cyan text-text-primary shadow-sm"
                        : "bg-surface-base/70 hover:bg-surface-card border-border-subtle hover:border-accent-blue/40 text-text-secondary hover:text-text-primary"
                    }`}
                  >
                    <div className="flex flex-col min-w-0">
                      <span className="text-xs font-medium truncate">
                        {isSelected && isDirty
                          ? `${preset.name} *`
                          : preset.name}
                      </span>
                      <span className="text-[9px] font-mono text-text-muted">
                        {preset.category}
                      </span>
                    </div>
                    {isSelected && (
                      <Check className="w-3.5 h-3.5 text-accent-cyan shrink-0 ml-1" />
                    )}
                  </button>
                );
              })}
            </div>
          </div>
        </div>

        {/* 4. Footer */}
        <div className="px-4 py-2 border-t border-border-subtle bg-surface-card/60 flex items-center justify-between text-[10px] text-text-muted font-mono">
          <span>Atomic persistence via staging .tmp + MoveFileExW</span>
          <kbd className="bg-surface-base px-1.5 py-0.5 rounded border border-border-subtle">
            Esc to close
          </kbd>
        </div>
      </div>
    </div>
  );
};
