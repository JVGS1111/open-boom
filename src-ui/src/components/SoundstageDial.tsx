import React from "react";
import { Compass } from "lucide-react";

interface SoundstageDialProps {
  enabled: boolean;
  intensity: number;
  onIntensityChange: (intensity: number) => void;
  onToggle: () => void;
}

export const SoundstageDial: React.FC<SoundstageDialProps> = ({
  enabled,
  intensity,
  onIntensityChange,
  onToggle,
}) => {
  // Arc calculation for circular dial (radius = 85, circumference = 2 * PI * 85 ~= 534)
  const radius = 85;
  const circumference = 2 * Math.PI * radius;
  const activeArcOffset = circumference - (circumference * intensity * 0.75); // 270 degree sweep

  const speakerNodes = [
    { label: "FL", angle: -45, x: 26, y: 35 },
    { label: "FC", angle: 0, x: 97, y: 12 },
    { label: "FR", angle: 45, x: 168, y: 35 },
    { label: "SL", angle: -90, x: 10, y: 97 },
    { label: "SR", angle: 90, x: 184, y: 97 },
    { label: "BL", angle: -135, x: 38, y: 160 },
    { label: "BR", angle: 135, x: 156, y: 160 },
    { label: "SUB", angle: 180, x: 97, y: 178 },
  ];

  return (
    <div className="flex flex-col items-center justify-center p-3 relative">
      <div className="relative w-52 h-52 flex items-center justify-center">
        {/* Virtual 7.1 Speaker Nodes Orbit */}
        {speakerNodes.map((node) => (
          <div
            key={node.label}
            style={{ left: `${node.x}px`, top: `${node.y}px` }}
            className={`absolute z-10 text-[9px] font-mono px-1 py-0.5 rounded transition-all duration-200 ${
              enabled
                ? "bg-surface-elevated text-accent-cyan border border-accent-cyan/40 shadow-cyan-glow-sm"
                : "bg-surface-card text-text-muted border border-border-subtle"
            }`}
          >
            {node.label}
          </div>
        ))}

        {/* Circular SVG Track */}
        <svg className="w-full h-full -rotate-90 pointer-events-none" viewBox="0 0 210 210">
          <circle
            cx="105"
            cy="105"
            r={radius}
            className="fill-none stroke-surface-card stroke-[7]"
          />
          <circle
            cx="105"
            cy="105"
            r={radius}
            strokeDasharray={circumference}
            strokeDashoffset={enabled ? activeArcOffset : circumference}
            className="fill-none stroke-accent-cyan stroke-[7] transition-all duration-150 [stroke-linecap:round] drop-shadow-[0_0_8px_rgba(56,189,248,0.35)]"
          />
        </svg>

        {/* Central Machined Rotary Knob */}
        <div
          onClick={onToggle}
          role="button"
          tabIndex={0}
          title={enabled ? "3D Spatializer Active - Click to Toggle" : "3D Spatializer Inactive - Click to Toggle"}
          className={`absolute w-32 h-32 rounded-full border flex flex-col items-center justify-center cursor-pointer transition-all duration-200 select-none ${
            enabled
              ? "bg-surface-elevated border-border-default shadow-[inset_0_2px_6px_rgba(255,255,255,0.08),0_8px_24px_rgba(0,0,0,0.4)]"
              : "bg-surface-base border-border-subtle opacity-70"
          }`}
        >
          <span className="text-[10px] font-semibold tracking-wider text-text-secondary uppercase">
            SURROUND 3D
          </span>
          <span className="font-display text-2xl font-bold text-text-primary my-0.5">
            {enabled ? `${Math.round(intensity * 100)}%` : "OFF"}
          </span>
          <span className="font-mono text-[9px] text-accent-cyan font-medium">
            BINAURAL HRTF
          </span>
        </div>
      </div>

      {/* Intensity Slider & Calibration */}
      <div className="flex items-center gap-3 mt-2">
        <input
          type="range"
          min="0"
          max="1"
          step="0.05"
          value={intensity}
          disabled={!enabled}
          onChange={(e) => onIntensityChange(parseFloat(e.target.value))}
          className="w-28 h-1 bg-border-subtle rounded appearance-none cursor-pointer accent-accent-cyan disabled:opacity-40"
        />
        <button
          type="button"
          onClick={onToggle}
          className="flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-medium bg-surface-card border border-border-subtle hover:border-border-default rounded text-text-secondary hover:text-text-primary transition-colors cursor-pointer"
        >
          <Compass className="w-3.5 h-3.5 text-accent-cyan" />
          <span>Calibrate Soundstage</span>
        </button>
      </div>
    </div>
  );
};
