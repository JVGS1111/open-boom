import React, { useRef, useState, useCallback } from "react";
import { Compass, Orbit } from "lucide-react";
import { useI18n } from "../i18n";

/**
 * Maps standard acoustic azimuth degrees to SVG Cartesian coordinates on the dial.
 *
 * Polar coordinates convention:
 * 0°    = North / Center (x = cx, y = cy - R)
 * +45°  = Front-Right (x = cx + R * sin(45°), y = cy - R * cos(45°))
 * +90°  = East / Right / Surround-Right (x = cx + R, y = cy)
 * +135° = Rear-Right
 * ±180° = South / Rear / LFE (x = cx, y = cy + R)
 * -90°  = West / Left / Surround-Left (x = cx - R, y = cy)
 * -45°  = Front-Left
 */
export const azimuthToCartesian = (
  azimuthDeg: number,
  r: number,
  originX: number = 96,
  originY: number = 96
) => {
  const safeAngle =
    typeof azimuthDeg === "number" && !isNaN(azimuthDeg) && isFinite(azimuthDeg)
      ? azimuthDeg
      : 0.0;
  const rad = (safeAngle * Math.PI) / 180.0;
  return {
    x: originX + r * Math.sin(rad),
    y: originY - r * Math.cos(rad),
  };
};

export interface SoundstageDialProps {
  enabled: boolean;
  intensity: number;
  onIntensityChange: (intensity: number) => void;
  onToggle: () => void;
  onOpenConfigurator?: () => void;
  orbitEnabled?: boolean;
  orbitPhaseDeg?: number;
  onToggleOrbit?: () => void;
}

export const SoundstageDial: React.FC<SoundstageDialProps> = ({
  enabled,
  intensity,
  onIntensityChange,
  onToggle,
  onOpenConfigurator,
  orbitEnabled = false,
  orbitPhaseDeg = 0.0,
  onToggleOrbit,
}) => {
  const { t } = useI18n();
  const dialRef = useRef<HTMLDivElement>(null);
  const [isDragging, setIsDragging] = useState(false);

  // Geometry constants adhering to UX-DR4
  // Outer size: 192px, Inner knob: 144px, Track width: 10px
  const radius = 84;
  const cx = 96;
  const cy = 96;

  // 300° total sweep starting at 120° (South-West) and ending at 420° / 60° (South-East)
  const startAngleDeg = 120;
  const totalSweepDeg = 300;

  // Convert polar coordinates to Cartesian SVG point
  const polarToCartesian = (angleDeg: number, r: number) => {
    const rad = (angleDeg * Math.PI) / 180.0;
    return {
      x: cx + r * Math.cos(rad),
      y: cy + r * Math.sin(rad),
    };
  };

  const clampedIntensity = Math.max(0.0, Math.min(1.0, intensity));
  const currentAngleDeg = startAngleDeg + clampedIntensity * totalSweepDeg;

  // Background track arc (full 300°)
  const pStart = polarToCartesian(startAngleDeg, radius);
  const pEnd = polarToCartesian(startAngleDeg + totalSweepDeg, radius);
  const bgArcPath = `M ${pStart.x.toFixed(2)} ${pStart.y.toFixed(2)} A ${radius} ${radius} 0 1 1 ${pEnd.x.toFixed(2)} ${pEnd.y.toFixed(2)}`;

  // Active cyan fill arc (0° to 300° based on intensity)
  const pCurr = polarToCartesian(currentAngleDeg, radius);
  const sweepAngle = clampedIntensity * totalSweepDeg;
  const largeArcFlag = sweepAngle > 180 ? 1 : 0;
  const activeArcPath =
    clampedIntensity > 0.005
      ? `M ${pStart.x.toFixed(2)} ${pStart.y.toFixed(2)} A ${radius} ${radius} 0 ${largeArcFlag} 1 ${pCurr.x.toFixed(2)} ${pCurr.y.toFixed(2)}`
      : "";

  // Detent pips at 25%, 50%, and 75%
  const detents = [
    { fraction: 0.25, label: "25%" },
    { fraction: 0.5, label: "50%" },
    { fraction: 0.75, label: "75%" },
  ];

  // Satellite speaker labels positioned around the perimeter
  const speakerNodes = [
    { label: "FL", angle: -45, x: 18, y: 32 },
    { label: "C", angle: 0, x: 96, y: 6 },
    { label: "FR", angle: 45, x: 174, y: 32 },
    { label: "SL", angle: -90, x: 8, y: 96 },
    { label: "SR", angle: 90, x: 184, y: 96 },
    { label: "RL", angle: -135, x: 30, y: 162 },
    { label: "RR", angle: 135, x: 162, y: 162 },
    { label: "LFE", angle: 180, x: 96, y: 186 },
  ];

  const intensityRef = useRef(clampedIntensity);
  intensityRef.current = clampedIntensity;

  // Polar drag physics: theta = atan2(y - y0, x - x0)
  const handlePointerCoord = useCallback(
    (clientX: number, clientY: number) => {
      if (!dialRef.current) return;
      const rect = dialRef.current.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;

      const dx = clientX - centerX;
      const dy = clientY - centerY;

      let thetaDeg = (Math.atan2(dy, dx) * 180) / Math.PI;
      if (thetaDeg < 0) {
        thetaDeg += 360;
      }

      // Compute relative angle clockwise from start angle (120°)
      let deltaAngle = (thetaDeg - startAngleDeg + 360) % 360;

      let nextFraction: number;
      if (deltaAngle <= totalSweepDeg) {
        nextFraction = deltaAngle / totalSweepDeg;
      } else {
        // In the 60° bottom dead-zone (totalSweepDeg to 360°):
        // Clamp to 0.0 or 1.0 depending on current position to prevent erratic flips
        nextFraction = intensityRef.current < 0.5 ? 0.0 : 1.0;
      }

      // Prevent jumping across stops when moving mouse fast near boundaries
      if (intensityRef.current > 0.75 && nextFraction < 0.25) {
        nextFraction = 1.0;
      } else if (intensityRef.current < 0.25 && nextFraction > 0.75) {
        nextFraction = 0.0;
      }

      const rounded = Math.round(nextFraction * 100) / 100;
      onIntensityChange(Math.max(0.0, Math.min(1.0, rounded)));
    },
    [onIntensityChange]
  );

  const handlePointerDown = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    setIsDragging(true);
    e.currentTarget.setPointerCapture(e.pointerId);
    handlePointerCoord(e.clientX, e.clientY);
  };

  const handlePointerMove = (e: React.PointerEvent) => {
    if (!isDragging) return;
    handlePointerCoord(e.clientX, e.clientY);
  };

  const handlePointerUp = (e: React.PointerEvent) => {
    if (isDragging) {
      setIsDragging(false);
      try {
        e.currentTarget.releasePointerCapture(e.pointerId);
      } catch {
        // Pointer capture cleanup
      }
    }
  };

  // Mouse wheel stepping: 2% per notch (5% with Shift)
  const handleWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    const step = e.shiftKey ? 0.05 : 0.02;
    const delta = e.deltaY < 0 ? step : -step;
    const nextVal = Math.max(0.0, Math.min(1.0, clampedIntensity + delta));
    onIntensityChange(Math.round(nextVal * 100) / 100);
  };

  // Double-click center knob snaps back to default 50%
  const handleDoubleClickKnob = (e: React.MouseEvent) => {
    e.stopPropagation();
    onIntensityChange(0.5);
  };

  // Keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    const step = e.shiftKey ? 0.05 : 0.02;
    if (e.key === "ArrowUp" || e.key === "ArrowRight") {
      e.preventDefault();
      onIntensityChange(Math.min(1.0, Math.round((clampedIntensity + step) * 100) / 100));
    } else if (e.key === "ArrowDown" || e.key === "ArrowLeft") {
      e.preventDefault();
      onIntensityChange(Math.max(0.0, Math.round((clampedIntensity - step) * 100) / 100));
    } else if (e.key === "Home") {
      e.preventDefault();
      onIntensityChange(0.0);
    } else if (e.key === "End") {
      e.preventDefault();
      onIntensityChange(1.0);
    } else if (e.key === "PageUp") {
      e.preventDefault();
      onIntensityChange(Math.min(1.0, Math.round((clampedIntensity + 0.1) * 100) / 100));
    } else if (e.key === "PageDown") {
      e.preventDefault();
      onIntensityChange(Math.max(0.0, Math.round((clampedIntensity - 0.1) * 100) / 100));
    } else if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      onToggle();
    }
  };

  return (
    <div className="flex flex-col items-center justify-center p-2 relative select-none">
      {/* 192px Outer Dial Container */}
      <div
        ref={dialRef}
        role="slider"
        aria-label="3D Surround Intensity"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(clampedIntensity * 100)}
        aria-disabled={!enabled}
        tabIndex={0}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onWheel={handleWheel}
        onKeyDown={handleKeyDown}
        className="relative w-[192px] h-[192px] flex items-center justify-center cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-accent-cyan/60 rounded-full"
      >
        {/* Virtual 7.1 Speaker Satellite Dots */}
        {speakerNodes.map((node) => (
          <div
            key={node.label}
            style={{
              left: `${node.x}px`,
              top: `${node.y}px`,
              transform: "translate(-50%, -50%)",
            }}
            className={`absolute z-10 text-[8px] font-mono font-bold px-1 py-0.2 rounded pointer-events-none transition-colors duration-200 ${
              enabled
                ? "bg-surface-elevated/90 text-accent-cyan border border-accent-cyan/30 shadow-[0_0_6px_rgba(56,189,248,0.25)]"
                : "bg-surface-card/60 text-text-muted/60 border border-border-subtle"
            }`}
          >
            {node.label}
          </div>
        ))}

        {/* 192px SVG Rotary Track & Glow */}
        <svg
          className="absolute inset-0 w-full h-full pointer-events-none"
          viewBox="0 0 192 192"
        >
          {/* Background Outer Track (300° arc, 10px width) */}
          <path
            d={bgArcPath}
            fill="none"
            stroke="currentColor"
            strokeWidth="10"
            strokeLinecap="round"
            className="text-surface-card stroke-surface-card"
          />

          {/* Active Cyan Fill Arc (0° to 300°) */}
          {enabled && activeArcPath && (
            <path
              d={activeArcPath}
              fill="none"
              stroke="#38bdf8"
              strokeWidth="10"
              strokeLinecap="round"
              className="drop-shadow-[0_0_8px_rgba(56,189,248,0.45)]"
            />
          )}

          {/* Glowing Beacon Dot at tip of arc */}
          {enabled && clampedIntensity > 0.01 && (
            <circle
              cx={pCurr.x.toFixed(2)}
              cy={pCurr.y.toFixed(2)}
              r="6"
              fill="#ffffff"
              stroke="#38bdf8"
              strokeWidth="2.5"
              className="drop-shadow-[0_0_10px_rgba(56,189,248,0.8)]"
            />
          )}

          {/* Detent Pips at 25%, 50%, 75% */}
          {detents.map((det) => {
            const pipAngle = startAngleDeg + det.fraction * totalSweepDeg;
            const pipPoint = polarToCartesian(pipAngle, radius);
            const isLit = enabled && clampedIntensity >= det.fraction;

            return (
              <circle
                key={det.label}
                cx={pipPoint.x.toFixed(2)}
                cy={pipPoint.y.toFixed(2)}
                r="3.5"
                fill={isLit ? "#38bdf8" : "#2e323e"}
                stroke={isLit ? "#ffffff" : "#1e2128"}
                strokeWidth="1"
                className={`transition-colors duration-150 ${
                  isLit ? "drop-shadow-[0_0_5px_rgba(56,189,248,0.6)]" : ""
                }`}
              />
            );
          })}

          {/* Luminous Neon 8D Orbital Trajectory Comet / Beacon */}
          {orbitEnabled && (
            <g data-testid="orbital-comet" className="orbital-comet pointer-events-none">
              {/* Planetary Orbital Guide Ring */}
              <circle
                cx={cx}
                cy={cy}
                r={radius}
                fill="none"
                stroke="#38bdf8"
                strokeWidth="1.5"
                strokeDasharray="3 4"
                opacity="0.35"
              />

              {/* Luminous Comet Tail Trail (Fading particles trailing behind) */}
              {(() => {
                const pHead = azimuthToCartesian(orbitPhaseDeg, radius, cx, cy);
                const pTail1 = azimuthToCartesian(orbitPhaseDeg - 7, radius, cx, cy);
                const pTail2 = azimuthToCartesian(orbitPhaseDeg - 14, radius, cx, cy);
                const pTail3 = azimuthToCartesian(orbitPhaseDeg - 21, radius, cx, cy);

                return (
                  <>
                    <circle
                      data-testid="comet-tail-3"
                      cx={pTail3.x.toFixed(2)}
                      cy={pTail3.y.toFixed(2)}
                      r="2"
                      fill="#38bdf8"
                      opacity="0.25"
                      className="drop-shadow-[0_0_4px_rgba(56,189,248,0.3)]"
                    />
                    <circle
                      data-testid="comet-tail-2"
                      cx={pTail2.x.toFixed(2)}
                      cy={pTail2.y.toFixed(2)}
                      r="3"
                      fill="#38bdf8"
                      opacity="0.5"
                      className="drop-shadow-[0_0_6px_rgba(56,189,248,0.5)]"
                    />
                    <circle
                      data-testid="comet-tail-1"
                      cx={pTail1.x.toFixed(2)}
                      cy={pTail1.y.toFixed(2)}
                      r="4"
                      fill="#38bdf8"
                      opacity="0.75"
                      className="drop-shadow-[0_0_8px_rgba(56,189,248,0.7)]"
                    />

                    {/* Outer Pulsing Halo */}
                    <circle
                      data-testid="comet-beacon-halo"
                      cx={pHead.x.toFixed(2)}
                      cy={pHead.y.toFixed(2)}
                      r="7.5"
                      fill="#38bdf8"
                      opacity="0.35"
                      className="animate-pulse"
                    />

                    {/* Luminous Neon Beacon Disc */}
                    <circle
                      data-testid="comet-beacon-disc"
                      cx={pHead.x.toFixed(2)}
                      cy={pHead.y.toFixed(2)}
                      r="5.5"
                      fill="#06b6d4"
                      stroke="#38bdf8"
                      strokeWidth="1.5"
                      className="drop-shadow-[0_0_12px_rgba(56,189,248,0.95)]"
                    />

                    {/* Intense Core Beacon Spark */}
                    <circle
                      data-testid="comet-beacon-head"
                      cx={pHead.x.toFixed(2)}
                      cy={pHead.y.toFixed(2)}
                      r="2.8"
                      fill="#ffffff"
                    />
                  </>
                );
              })()}
            </g>
          )}
        </svg>

        {/* 144px Inner Machined Rotary Knob */}
        <div
          onDoubleClick={handleDoubleClickKnob}
          onClick={onToggle}
          title={
            enabled
              ? "Surround 3D Active. Click to power off, Double-click to snap 50%"
              : "Surround 3D Inactive. Click to power on"
          }
          className={`absolute w-[144px] h-[144px] rounded-full border flex flex-col items-center justify-center cursor-pointer transition-all duration-200 select-none ${
            enabled
              ? "bg-surface-elevated border-border-default shadow-[inset_0_2px_8px_rgba(255,255,255,0.06),0_8px_24px_rgba(0,0,0,0.5)] hover:border-accent-cyan/40"
              : "bg-surface-base border-border-subtle opacity-70"
          }`}
        >
          <span className="text-[10px] font-bold tracking-wider text-text-secondary uppercase">
            {t.soundstage.title}
          </span>
          <span
            className={`font-mono text-3xl font-extrabold my-0.5 tracking-tight ${
              enabled ? "text-accent-cyan drop-shadow-[0_0_8px_rgba(56,189,248,0.3)]" : "text-text-muted"
            }`}
          >
            {enabled ? `${Math.round(clampedIntensity * 100)}%` : t.common.off.toUpperCase()}
          </span>
          <span
            className={`font-mono text-[9px] font-medium tracking-wide ${
              orbitEnabled
                ? "text-accent-cyan font-bold drop-shadow-[0_0_6px_rgba(56,189,248,0.5)]"
                : "text-text-muted"
            }`}
          >
            {orbitEnabled ? "8D ORBIT ACTIVE" : "KEMAR 7.1 HRTF"}
          </span>
        </div>
      </div>

      {/* Control Actions & Soundstage Configurator launcher */}
      <div className="flex flex-col items-center gap-2 mt-3 w-full">
        <div className="flex items-center justify-center gap-2 w-full">
          <button
            type="button"
            onClick={onToggle}
            className={`px-2.5 py-1 text-[11px] font-semibold rounded border transition-colors cursor-pointer ${
              enabled
                ? "bg-accent-cyan/15 text-accent-cyan border-accent-cyan/40 hover:bg-accent-cyan/25"
                : "bg-surface-card text-text-muted border-border-subtle hover:text-text-primary"
            }`}
          >
            {enabled ? t.common.active : t.common.bypassed}
          </button>

          <button
            type="button"
            data-testid="orbit-8d-button"
            aria-pressed={orbitEnabled}
            aria-label={t.soundstage.orbit8D || "8D Orbit"}
            onClick={onToggleOrbit}
            className={`flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-semibold rounded border transition-all duration-150 cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-accent-cyan/60 ${
              orbitEnabled
                ? "bg-accent-cyan/20 text-accent-cyan border-accent-cyan shadow-[0_0_12px_rgba(56,189,248,0.45)] ring-1 ring-accent-cyan/40"
                : "bg-surface-card text-text-muted border-border-subtle hover:text-text-primary hover:border-border-default"
            }`}
          >
            <Orbit
              className={`w-3.5 h-3.5 ${
                orbitEnabled ? "text-accent-cyan animate-spin" : "text-text-muted"
              }`}
              style={{ animationDuration: "6s" }}
            />
            <span>{t.soundstage.orbit8D || "8D Orbit"}</span>
          </button>
        </div>

        <button
          type="button"
          onClick={onOpenConfigurator}
          title="Open 8-Channel Polar Soundstage Configurator (Ctrl + Shift + S)"
          className="flex items-center justify-center gap-1.5 px-3 py-1 text-[11px] font-medium bg-surface-card border border-border-subtle hover:border-accent-cyan/40 hover:text-accent-cyan rounded text-text-secondary transition-all cursor-pointer shadow-sm w-full max-w-[200px]"
        >
          <Compass className="w-3.5 h-3.5 text-accent-cyan" />
          <span>{t.soundstage.calibrateSoundstage}</span>
        </button>
      </div>
    </div>
  );
};
