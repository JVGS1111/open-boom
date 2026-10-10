import React from "react";
import { SlidersHorizontal, Activity } from "lucide-react";

interface EqualizerPreviewProps {
  activePreset: string;
  bands: number[];
  preampOffsetDb: number;
}

export const EqualizerPreview: React.FC<EqualizerPreviewProps> = ({
  activePreset,
  bands,
  preampOffsetDb,
}) => {
  return (
    <div className="bg-surface-card border border-border-subtle rounded-lg p-3.5 flex flex-col justify-between h-full">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <SlidersHorizontal className="w-4 h-4 text-accent-blue" />
          <span className="font-semibold text-xs tracking-wide text-text-primary">
            DUAL-STAGE EQUALIZER
          </span>
        </div>
        <div className="flex items-center gap-1.5 text-[10px] font-mono text-text-secondary bg-surface-base px-2 py-0.5 rounded border border-border-subtle">
          <span>{activePreset}</span>
          <span className="text-accent-blue">({preampOffsetDb.toFixed(1)} dB)</span>
        </div>
      </div>

      {/* Visual EQ Curve Faders Preview */}
      <div className="h-20 bg-surface-base/80 rounded border border-border-subtle p-2 flex items-end justify-between gap-0.5 relative overflow-hidden">
        {/* Center 0 dB Reference Grid Line */}
        <div className="absolute top-1/2 left-0 right-0 h-px bg-border-default/50 pointer-events-none" />

        {bands.map((gain, idx) => {
          // Normalize -12dB to +12dB to 0% - 100% height
          const normalized = Math.max(10, Math.min(90, 50 + (gain / 12) * 40));
          const isBoost = gain > 0;
          return (
            <div
              key={idx}
              className="flex-1 flex flex-col items-center justify-end h-full group cursor-pointer"
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
        <div className="flex items-center gap-1">
          <Activity className="w-3 h-3 text-accent-cyan" />
          <span>AutoEq: Harman Over-Ear Target</span>
        </div>
        <span className="font-mono">31 BANDS / DF-II</span>
      </div>
    </div>
  );
};
