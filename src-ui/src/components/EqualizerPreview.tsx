import React from "react";
import { SlidersHorizontal, Activity, Maximize2, ChevronDown } from "lucide-react";
import { formatPresetDisplayLabel } from "./PresetSelectorPopover";

export interface EqualizerPreviewProps {
  activePreset: string;
  isPresetDirty?: boolean;
  bands: number[];
  preampOffsetDb: number;
  onExpand?: () => void;
  onOpenPresets?: () => void;
  activeAutoEqName?: string;
  onOpenAutoEq?: () => void;
}

export const EqualizerPreview: React.FC<EqualizerPreviewProps> = ({
  activePreset,
  isPresetDirty = false,
  bands,
  preampOffsetDb,
  onExpand,
  onOpenPresets,
  activeAutoEqName,
  onOpenAutoEq,
}) => {
  const presetLabel = formatPresetDisplayLabel(activePreset, isPresetDirty);

  return (
    <div className="bg-surface-card border border-border-subtle rounded-lg p-3.5 flex flex-col justify-between h-full relative">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <SlidersHorizontal className="w-4 h-4 text-accent-blue" />
          <span className="font-semibold text-xs tracking-wide text-text-primary">
            DUAL-STAGE EQUALIZER
          </span>
        </div>

        <div className="flex items-center gap-2">
          {/* Expand EQ Pill Button with Hotkey Badge (UX-DR7, EXPERIENCE.md §4.5) */}
          <button
            type="button"
            onClick={onExpand}
            className="flex items-center gap-1.5 px-2 py-0.5 rounded text-[10px] font-medium bg-accent-blue/15 hover:bg-accent-blue/25 text-accent-cyan hover:text-white border border-accent-blue/30 transition-all cursor-pointer group"
            title="Open Equalizer Studio Drawer (Ctrl + Shift + E)"
          >
            <Maximize2 className="w-3 h-3 text-accent-cyan group-hover:scale-110 transition-transform" />
            <span>Expand EQ</span>
            <kbd className="text-[9px] font-mono text-text-muted group-hover:text-text-secondary bg-surface-base px-1 rounded border border-border-subtle">
              Ctrl+Shift+E
            </kbd>
          </button>

          {/* Preset Selector Pill Button with Dirty Asterisk Indicator (FR11, EXPERIENCE.md §5.2) */}
          <button
            type="button"
            onClick={onOpenPresets}
            title="Select Factory Genre Preset or Save Custom Preset"
            className={`flex items-center gap-1.5 text-[10px] font-mono px-2 py-0.5 rounded border transition-all cursor-pointer ${
              isPresetDirty
                ? "text-accent-cyan bg-accent-blue/15 border-accent-cyan/50 hover:bg-accent-blue/25"
                : "text-text-secondary hover:text-text-primary bg-surface-base hover:bg-surface-overlay border-border-subtle hover:border-accent-blue/40"
            }`}
          >
            <span>{presetLabel}</span>
            <span className="text-accent-blue">({preampOffsetDb.toFixed(1)} dB)</span>
            <ChevronDown className="w-3 h-3 text-text-muted" />
          </button>
        </div>
      </div>

      {/* Visual EQ Curve Faders Preview (Clickable trigger to expand drawer) */}
      <div
        onClick={onExpand}
        className="h-20 bg-surface-base/80 hover:bg-surface-base rounded border border-border-subtle hover:border-accent-blue/40 p-2 flex items-end justify-between gap-0.5 relative overflow-hidden cursor-pointer transition-colors group"
        title="Click to expand Equalizer Studio Drawer"
      >
        {/* Center 0 dB Reference Grid Line */}
        <div className="absolute top-1/2 left-0 right-0 h-px bg-border-default/50 pointer-events-none" />

        {bands.map((gain, idx) => {
          // Normalize -12dB to +12dB to 0% - 100% height
          const normalized = Math.max(10, Math.min(90, 50 + (gain / 12) * 40));
          const isBoost = gain > 0;
          return (
            <div
              key={idx}
              className="flex-1 flex flex-col items-center justify-end h-full"
              title={`Band ${idx + 1}: ${gain > 0 ? "+" : ""}${gain.toFixed(1)} dB`}
            >
              <div
                style={{ height: `${normalized}%` }}
                className={`w-full max-w-[6px] rounded-t-sm transition-all duration-150 ${
                  isBoost
                    ? "bg-accent-blue group-hover:bg-accent-cyan shadow-[0_0_4px_rgba(59,130,246,0.5)]"
                    : "bg-border-default group-hover:bg-text-secondary"
                }`}
              />
            </div>
          );
        })}
      </div>

      <div className="flex items-center justify-between mt-2 pt-2 border-t border-border-subtle/60 text-[10px] text-text-muted">
        <button
          type="button"
          onClick={onOpenAutoEq}
          className="flex items-center gap-1.5 px-1.5 py-0.5 rounded hover:bg-surface-base text-text-muted hover:text-accent-cyan border border-transparent hover:border-accent-blue/30 transition-all cursor-pointer group"
          title="Search 4,000+ AutoEq Headphone Profiles"
        >
          <Activity className="w-3 h-3 text-accent-cyan group-hover:scale-110 transition-transform" />
          <span>
            AutoEq:{" "}
            <strong className="font-semibold text-text-primary group-hover:text-accent-cyan">
              {activeAutoEqName || "Select Headphone"}
            </strong>
          </span>
        </button>
        <span className="font-mono">31 BANDS / DF-II</span>
      </div>
    </div>
  );
};
