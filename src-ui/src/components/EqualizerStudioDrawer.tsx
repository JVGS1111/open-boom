import React, { useEffect, useRef, useState, useCallback } from "react";
import { SlidersHorizontal, RotateCcw, X, Check, Activity } from "lucide-react";

export const ISO_31_FREQUENCIES: number[] = [
  20, 25, 31.5, 40, 50, 63, 80, 100, 125, 160, 200, 250, 315, 400,
  500, 630, 800, 1000, 1250, 1600, 2000, 2500, 3150, 4000, 5000, 6300,
  8000, 10000, 12500, 16000, 20000,
];

export const ISO_10_FREQUENCIES: number[] = [
  31.5, 63, 125, 250, 500, 1000, 2000, 4000, 8000, 16000,
];

export const ISO_10_TO_31_INDICES: number[] = [
  2, 5, 8, 11, 14, 17, 20, 23, 26, 29,
];

export interface Point {
  x: number;
  y: number;
}

export interface BezierSegment {
  p0: Point;
  cp1: Point;
  cp2: Point;
  p1: Point;
}

/**
 * Format frequency in Hertz to concise engineering label (e.g., 31.5, 1k, 16k).
 */
export function formatFrequency(hz: number): string {
  if (hz >= 1000) {
    const khz = hz / 1000;
    return `${parseFloat(khz.toFixed(2))}k`;
  }
  return hz % 1 === 0 ? `${hz}` : `${hz.toFixed(1)}`;
}

/**
 * Soft magnetic detent snap within +/-0.3 dB of 0.0 dB and [-12.0, +12.0] dB clamping.
 */
export function applyMagneticSnap(gainDb: number, detentThreshold: number = 0.3): number {
  if (Math.abs(gainDb) <= detentThreshold) {
    return 0.0;
  }
  const clamped = Math.max(-12.0, Math.min(12.0, gainDb));
  return Math.round(clamped * 10) / 10;
}

/**
 * Fader movement delta calculator with 5x fine precision scaling when Shift is held.
 */
export function calculateFaderDelta(
  deltaYPixels: number,
  isShiftHeld: boolean,
  trackHeightPx: number = 180,
  totalRangeDb: number = 24.0
): number {
  const baseSensitivity = totalRangeDb / trackHeightPx;
  const sensitivity = isShiftHeld ? baseSensitivity / 5.0 : baseSensitivity;
  return -deltaYPixels * sensitivity;
}

/**
 * Logarithmic frequency mapping to [0, width] for 20 Hz - 20,000 Hz spectrum.
 */
export function frequencyToLogX(
  freqHz: number,
  width: number,
  minFreq: number = 20,
  maxFreq: number = 20000
): number {
  const logMin = Math.log10(minFreq);
  const logMax = Math.log10(maxFreq);
  const logF = Math.log10(Math.max(minFreq, Math.min(maxFreq, freqHz)));
  const normalized = (logF - logMin) / (logMax - logMin);
  return normalized * width;
}

/**
 * Gain mapping in [-12, +12] dB to Y coordinate in [0, height].
 */
export function gainToY(
  gainDb: number,
  height: number,
  minGain: number = -12.0,
  maxGain: number = 12.0
): number {
  const clamped = Math.max(minGain, Math.min(maxGain, gainDb));
  const normalized = (clamped - minGain) / (maxGain - minGain);
  return (1 - normalized) * height;
}

/**
 * Convert 31 coordinate points into smooth Catmull-Rom cubic Bezier spline segments.
 */
export function computeSplineSegments(points: Point[]): BezierSegment[] {
  if (points.length < 2) return [];
  const segments: BezierSegment[] = [];

  for (let i = 0; i < points.length - 1; i++) {
    const p0 = points[i];
    const p1 = points[i + 1];
    const pPrev = i > 0 ? points[i - 1] : points[i];
    const pNext = i < points.length - 2 ? points[i + 2] : points[i + 1];

    const cp1: Point = {
      x: p0.x + (p1.x - pPrev.x) / 6,
      y: p0.y + (p1.y - pPrev.y) / 6,
    };
    const cp2: Point = {
      x: p1.x - (pNext.x - p0.x) / 6,
      y: p1.y - (pNext.y - p0.y) / 6,
    };

    segments.push({ p0, cp1, cp2, p1 });
  }

  return segments;
}

/**
 * Real-time Bezier spline canvas rendering procedure.
 */
export function renderSplineCanvas(
  canvas: HTMLCanvasElement,
  gains: number[],
  mode: "10" | "31"
) {
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const dpr = typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1;
  const width = canvas.clientWidth || 700;
  const height = canvas.clientHeight || 130;

  const targetW = Math.round(width * dpr);
  const targetH = Math.round(height * dpr);
  if (canvas.width !== targetW || canvas.height !== targetH) {
    canvas.width = targetW;
    canvas.height = targetH;
  }

  ctx.save();
  ctx.scale(dpr, dpr);
  ctx.clearRect(0, 0, width, height);

  // Background
  ctx.fillStyle = "#16171b";
  ctx.fillRect(0, 0, width, height);

  const paddingLeft = 44;
  const paddingRight = 16;
  const paddingTop = 12;
  const paddingBottom = 22;

  const plotWidth = width - paddingLeft - paddingRight;
  const plotHeight = height - paddingTop - paddingBottom;

  if (plotWidth <= 0 || plotHeight <= 0) {
    ctx.restore();
    return;
  }

  // Draw Horizontal Gain Gridlines (-12 dB, -6 dB, 0 dB, +6 dB, +12 dB)
  const gridGains = [12, 6, 0, -6, -12];
  ctx.textAlign = "right";
  ctx.textBaseline = "middle";
  ctx.font = "9px 'Cascadia Code', monospace";

  gridGains.forEach((g) => {
    const y = paddingTop + plotHeight * (0.5 - g / 24.0);
    const isZero = g === 0;

    ctx.beginPath();
    ctx.moveTo(paddingLeft, y);
    ctx.lineTo(paddingLeft + plotWidth, y);
    ctx.strokeStyle = isZero ? "rgba(56, 189, 248, 0.45)" : "rgba(46, 50, 62, 0.4)";
    ctx.lineWidth = isZero ? 1.5 : 1;
    if (!isZero) {
      ctx.setLineDash([3, 3]);
    } else {
      ctx.setLineDash([]);
    }
    ctx.stroke();
    ctx.setLineDash([]);

    // Gain label
    ctx.fillStyle = isZero ? "#38bdf8" : "#64748b";
    ctx.fillText(`${g > 0 ? "+" : ""}${g} dB`, paddingLeft - 6, y);
  });

  // Draw Vertical Frequency Gridlines (ISO Octaves)
  const keyFreqs = [31.5, 63, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];
  ctx.textAlign = "center";
  ctx.textBaseline = "top";

  keyFreqs.forEach((f) => {
    const u = (Math.log10(f) - Math.log10(20)) / 3.0;
    const x = paddingLeft + plotWidth * u;

    ctx.beginPath();
    ctx.moveTo(x, paddingTop);
    ctx.lineTo(x, paddingTop + plotHeight);
    ctx.strokeStyle = "rgba(46, 50, 62, 0.35)";
    ctx.lineWidth = 1;
    ctx.setLineDash([2, 4]);
    ctx.stroke();
    ctx.setLineDash([]);

    // Frequency label at bottom
    ctx.fillStyle = "#64748b";
    ctx.fillText(formatFrequency(f), x, paddingTop + plotHeight + 5);
  });

  // Calculate 31 band points across logarithmic frequency scale
  const points: Point[] = ISO_31_FREQUENCIES.map((f, idx) => {
    const u = (Math.log10(f) - Math.log10(20)) / 3.0;
    const x = paddingLeft + plotWidth * u;
    const gain = gains[idx] ?? 0;
    const clampedGain = Math.max(-12, Math.min(12, gain));
    const y = paddingTop + plotHeight * (0.5 - clampedGain / 24.0);
    return { x, y };
  });

  // Compute smooth Catmull-Rom cubic Bezier segments
  const segments = computeSplineSegments(points);

  // 1. Draw Cyan Gradient Fill under curve to 0 dB baseline
  const zeroY = paddingTop + plotHeight * 0.5;
  if (segments.length > 0) {
    ctx.beginPath();
    ctx.moveTo(points[0].x, points[0].y);
    segments.forEach((seg) => {
      ctx.bezierCurveTo(seg.cp1.x, seg.cp1.y, seg.cp2.x, seg.cp2.y, seg.p1.x, seg.p1.y);
    });
    ctx.lineTo(points[points.length - 1].x, zeroY);
    ctx.lineTo(points[0].x, zeroY);
    ctx.closePath();

    const fillGrad = ctx.createLinearGradient(0, paddingTop, 0, paddingTop + plotHeight);
    fillGrad.addColorStop(0, "rgba(56, 189, 248, 0.35)");
    fillGrad.addColorStop(0.5, "rgba(59, 130, 246, 0.15)");
    fillGrad.addColorStop(1, "rgba(56, 189, 248, 0.02)");
    ctx.fillStyle = fillGrad;
    ctx.fill();
  }

  // 2. Draw Cyan Spline Stroke with Neon Glow
  if (segments.length > 0) {
    ctx.beginPath();
    ctx.moveTo(points[0].x, points[0].y);
    segments.forEach((seg) => {
      ctx.bezierCurveTo(seg.cp1.x, seg.cp1.y, seg.cp2.x, seg.cp2.y, seg.p1.x, seg.p1.y);
    });
    ctx.strokeStyle = "#38bdf8";
    ctx.lineWidth = 2.5;
    ctx.shadowColor = "rgba(56, 189, 248, 0.75)";
    ctx.shadowBlur = 8;
    ctx.stroke();
    ctx.shadowBlur = 0;
  }

  // 3. Draw Frequency Node Dots
  points.forEach((pt, idx) => {
    const isOctave = ISO_10_TO_31_INDICES.includes(idx);
    const gain = gains[idx] ?? 0;
    const isNonZero = Math.abs(gain) > 0.05;

    const isHighlighted = mode === "10" ? isOctave : isNonZero || isOctave;
    const radius = isHighlighted ? 3.5 : 2;

    ctx.beginPath();
    ctx.arc(pt.x, pt.y, radius, 0, Math.PI * 2);
    if (isHighlighted) {
      ctx.fillStyle = isNonZero ? "#38bdf8" : "#94a3b8";
      ctx.shadowColor = "rgba(56, 189, 248, 0.6)";
      ctx.shadowBlur = 6;
      ctx.fill();
      ctx.shadowBlur = 0;
      ctx.strokeStyle = "#16171b";
      ctx.lineWidth = 1;
      ctx.stroke();
    } else {
      ctx.fillStyle = "rgba(100, 116, 139, 0.5)";
      ctx.fill();
    }
  });

  ctx.restore();
}

/**
 * Individual tactile Equalizer Fader with 180px track, Cascadia Code floating tooltip,
 * 5x precision with Shift, magnetic detent snap, and double-click reset.
 */
interface EqFaderProps {
  bandIndex: number;
  frequencyHz: number;
  gainDb: number;
  onChange: (newGainDb: number) => void;
  isCompact?: boolean;
}

export const EqFader: React.FC<EqFaderProps> = ({
  bandIndex,
  frequencyHz,
  gainDb,
  onChange,
  isCompact = false,
}) => {
  const [isDragging, setIsDragging] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const [showTooltip, setShowTooltip] = useState(false);
  const tooltipTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const trackRef = useRef<HTMLDivElement>(null);
  const lastYRef = useRef(0);
  const currentGainRef = useRef(gainDb);

  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    e.preventDefault();
    setIsDragging(true);
    setShowTooltip(true);
    if (tooltipTimerRef.current) {
      clearTimeout(tooltipTimerRef.current);
    }
    lastYRef.current = e.clientY;
    currentGainRef.current = gainDb;

    const target = e.currentTarget;
    try {
      target.setPointerCapture(e.pointerId);
    } catch {}

    const onPointerMove = (ev: PointerEvent) => {
      const deltaY = ev.clientY - lastYRef.current;
      lastYRef.current = ev.clientY;
      const deltaGain = calculateFaderDelta(deltaY, ev.shiftKey, 180, 24.0);
      const raw = Math.max(-12.0, Math.min(12.0, currentGainRef.current + deltaGain));
      currentGainRef.current = raw;
      const nextGain = ev.shiftKey
        ? Math.round(raw * 10) / 10
        : applyMagneticSnap(raw, 0.3);
      onChange(nextGain);
    };

    const onPointerUp = (ev: PointerEvent) => {
      setIsDragging(false);
      try {
        target.releasePointerCapture(ev.pointerId);
      } catch {}
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
      window.removeEventListener("pointercancel", onPointerUp);

      tooltipTimerRef.current = setTimeout(() => {
        setShowTooltip(false);
      }, 300);
    };

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("pointercancel", onPointerUp);
  };

  const handleDoubleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onChange(0.0);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    let next = gainDb;
    if (e.key === "ArrowUp" || e.key === "ArrowRight") {
      e.preventDefault();
      const step = e.shiftKey ? 0.1 : 0.5;
      next = e.shiftKey
        ? Math.round((gainDb + step) * 10) / 10
        : applyMagneticSnap(gainDb + step, 0.3);
    } else if (e.key === "ArrowDown" || e.key === "ArrowLeft") {
      e.preventDefault();
      const step = e.shiftKey ? 0.1 : 0.5;
      next = e.shiftKey
        ? Math.round((gainDb - step) * 10) / 10
        : applyMagneticSnap(gainDb - step, 0.3);
    } else if (e.key === "PageUp") {
      e.preventDefault();
      next = applyMagneticSnap(gainDb + 3.0, 0.3);
    } else if (e.key === "PageDown") {
      e.preventDefault();
      next = applyMagneticSnap(gainDb - 3.0, 0.3);
    } else if (e.key === "Home") {
      e.preventDefault();
      next = -12.0;
    } else if (e.key === "End") {
      e.preventDefault();
      next = 12.0;
    } else if (e.key === "0") {
      e.preventDefault();
      next = 0.0;
    }
    onChange(Math.max(-12.0, Math.min(12.0, next)));
  };

  // 180px track vertical geometry:
  // gain in [-12, +12] mapped to fraction [0, 1] (0 at -12 dB, 1 at +12 dB)
  const fraction = Math.max(0, Math.min(1, (gainDb + 12.0) / 24.0));
  // Y coordinate from top of 180px track: 0px at +12 dB, 90px at 0 dB, 180px at -12 dB
  const thumbY = Math.round((1.0 - fraction) * 180.0);

  const isBoost = gainDb > 0.05;
  const isCut = gainDb < -0.05;

  return (
    <div
      className={`flex flex-col items-center justify-between select-none relative group ${
        isCompact ? "w-[24px] min-w-[20px]" : "flex-1 max-w-[64px]"
      }`}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* Top Frequency Label */}
      <span
        className={`text-[10px] font-mono font-medium transition-colors ${
          isDragging || isHovered
            ? "text-accent-cyan font-bold"
            : isBoost
            ? "text-text-primary"
            : "text-text-secondary"
        }`}
        title={`Band ${bandIndex + 1}: ${frequencyHz} Hz`}
      >
        {formatFrequency(frequencyHz)}
      </span>

      {/* 180px Vertical Track Container */}
      <div
        ref={trackRef}
        role="slider"
        aria-label={`EQ Band ${bandIndex + 1} (${frequencyHz} Hz)`}
        aria-orientation="vertical"
        aria-valuemin={-12.0}
        aria-valuemax={12.0}
        aria-valuenow={gainDb}
        aria-valuetext={`${gainDb > 0 ? "+" : ""}${gainDb.toFixed(1)} dB`}
        tabIndex={0}
        onKeyDown={handleKeyDown}
        onPointerDown={handlePointerDown}
        onDoubleClick={handleDoubleClick}
        className="h-[180px] w-full flex items-center justify-center relative my-1 cursor-ns-resize touch-none focus:outline-none focus-visible:ring-1 focus-visible:ring-accent-cyan"
      >
        {/* Recessed Track Well */}
        <div className="w-[4px] h-[180px] bg-surface-base border border-border-subtle rounded-full relative overflow-hidden shadow-inner">
          {/* Active Fill from 0 dB center */}
          {isBoost && (
            <div
              style={{
                top: `${thumbY}px`,
                height: `${90 - thumbY}px`,
              }}
              className="absolute left-0 right-0 bg-accent-cyan shadow-[0_0_6px_rgba(56,189,248,0.5)]"
            />
          )}
          {isCut && (
            <div
              style={{
                top: `90px`,
                height: `${thumbY - 90}px`,
              }}
              className="absolute left-0 right-0 bg-accent-blue/80"
            />
          )}
        </div>

        {/* 0 dB Center Reference Detent Tick */}
        <div className="absolute top-[90px] w-[10px] h-[1.5px] bg-border-default pointer-events-none -translate-y-1/2" />

        {/* Fader Thumb (Cap) */}
        <div
          style={{ top: `${thumbY}px` }}
          className={`absolute left-1/2 -translate-x-1/2 -translate-y-1/2 transition-shadow cursor-grab active:cursor-grabbing ${
            isCompact ? "w-[16px] h-[10px]" : "w-[20px] h-[11px]"
          } rounded-[3px] bg-text-primary border ${
            isDragging
              ? "border-accent-cyan shadow-[0_0_8px_rgba(56,189,248,0.6)]"
              : isHovered
              ? "border-accent-cyan/80 shadow-md"
              : "border-border-default shadow-sm"
          } flex flex-col items-center justify-center gap-[1.5px]`}
        >
          {/* Tactile Grip Ridges */}
          <div className="w-[8px] h-[1px] bg-slate-400" />
          <div className="w-[8px] h-[1px] bg-slate-400" />
        </div>

        {/* Floating Cascadia Code Tooltip during Drag / Hover / 300ms delay */}
        {(isDragging || showTooltip || (isHovered && Math.abs(gainDb) > 0.05)) && (
          <div
            style={{
              top: `${Math.max(10, Math.min(160, thumbY))}px`,
            }}
            className="absolute -right-1 translate-x-full -translate-y-1/2 z-30 pointer-events-none px-1.5 py-0.5 rounded bg-surface-overlay border border-accent-cyan text-accent-cyan text-[10px] font-mono font-bold whitespace-nowrap shadow-xl"
          >
            {gainDb > 0 ? `+${gainDb.toFixed(1)}` : gainDb.toFixed(1)} dB
          </div>
        )}
      </div>

      {/* Bottom Cascadia Code dB Value Readout */}
      <span
        className={`text-[9.5px] font-mono transition-colors ${
          isDragging || isHovered
            ? "text-accent-cyan font-bold"
            : isBoost
            ? "text-accent-cyan font-semibold"
            : isCut
            ? "text-accent-blue"
            : "text-text-muted"
        }`}
      >
        {gainDb > 0 ? `+${gainDb.toFixed(1)}` : gainDb.toFixed(1)}
      </span>
    </div>
  );
};

export interface EqualizerStudioDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  mode: "10" | "31";
  onModeChange: (mode: "10" | "31") => void;
  gains: number[];
  onGainChange: (bandIndex: number, gainDb: number) => void;
  onResetAll: () => void;
  activePreset?: string;
  preampOffsetDb?: number;
}

export const EqualizerStudioDrawer: React.FC<EqualizerStudioDrawerProps> = ({
  isOpen,
  onClose,
  mode,
  onModeChange,
  gains,
  onGainChange,
  onResetAll,
  activePreset = "Harman Acoustic",
  preampOffsetDb = 0.0,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Redraw Bezier spline curve canvas whenever gains or mode change
  const redrawCanvas = useCallback(() => {
    if (canvasRef.current) {
      renderSplineCanvas(canvasRef.current, gains, mode);
    }
  }, [gains, mode]);

  useEffect(() => {
    redrawCanvas();
  }, [redrawCanvas]);

  // Handle ResizeObserver for high-DPI scaling
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const observer = new ResizeObserver(() => {
      redrawCanvas();
    });
    observer.observe(canvas);

    return () => observer.disconnect();
  }, [redrawCanvas]);

  // Keyboard shortcut listener: Escape key dismisses drawer cleanly
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  // Focus drawer container upon expanding
  useEffect(() => {
    if (isOpen) {
      containerRef.current?.focus();
    }
  }, [isOpen]);

  const activeIndices =
    mode === "10"
      ? ISO_10_TO_31_INDICES
      : Array.from({ length: 31 }, (_, i) => i);

  return (
    <div
      ref={containerRef}
      tabIndex={-1}
      aria-hidden={!isOpen}
      aria-label="Equalizer Studio Drawer"
      style={!isOpen ? { display: "none" } : undefined}
      className={`absolute inset-0 z-40 bg-surface-base/95 backdrop-blur-md border border-border-default rounded-xl shadow-2xl flex flex-col justify-between overflow-hidden transition-all duration-300 ease-out transform ${
        isOpen
          ? "translate-y-0 opacity-100 pointer-events-auto"
          : "translate-y-full opacity-0 pointer-events-none"
      }`}
    >
      {/* 1. Header Bar: Title, Mode Toggle, Reset & Close */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border-subtle bg-surface-card/60">
        <div className="flex items-center gap-2.5">
          <div className="p-1.5 rounded-lg bg-accent-blue/15 text-accent-cyan border border-accent-blue/30">
            <SlidersHorizontal className="w-4 h-4" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-sm font-semibold tracking-wide text-text-primary">
                EQUALIZER STUDIO
              </h2>
              <span className="text-[10px] font-mono text-accent-cyan bg-surface-base px-2 py-0.5 rounded border border-border-subtle">
                {activePreset}
              </span>
              {preampOffsetDb !== 0.0 && (
                <span className="text-[10px] font-mono text-text-muted">
                  Preamp: {preampOffsetDb.toFixed(1)} dB
                </span>
              )}
            </div>
            <p className="text-[11px] text-text-muted">
              Robert Bristow-Johnson Audio EQ peaking filters (Q = 4.31, DF-II Transposed)
            </p>
          </div>
        </div>

        {/* Center: 10-Band vs 31-Band Segmented Mode Toggle (FR12) */}
        <div className="flex items-center bg-surface-base p-0.5 rounded-lg border border-border-subtle">
          <button
            type="button"
            onClick={() => onModeChange("10")}
            className={`px-3 py-1 text-xs font-medium rounded-md transition-all cursor-pointer ${
              mode === "10"
                ? "bg-accent-blue text-white shadow-sm font-semibold"
                : "text-text-secondary hover:text-text-primary"
            }`}
          >
            10-Band
          </button>
          <button
            type="button"
            onClick={() => onModeChange("31")}
            className={`px-3 py-1 text-xs font-medium rounded-md transition-all cursor-pointer ${
              mode === "31"
                ? "bg-accent-blue text-white shadow-sm font-semibold"
                : "text-text-secondary hover:text-text-primary"
            }`}
          >
            31-Band
          </button>
        </div>

        {/* Right: Reset & Close buttons */}
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={onResetAll}
            title="Reset all bands to 0.0 dB"
            className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium bg-surface-card border border-border-subtle hover:border-border-default rounded-md text-text-secondary hover:text-text-primary transition-all cursor-pointer"
          >
            <RotateCcw className="w-3.5 h-3.5" />
            <span>Flat (0 dB)</span>
          </button>

          <button
            type="button"
            onClick={onClose}
            className="flex items-center gap-1.5 px-3.5 py-1 text-xs font-semibold bg-accent-cyan text-background hover:bg-accent-cyan/90 rounded-md transition-all shadow-cyan-glow-sm cursor-pointer"
          >
            <Check className="w-3.5 h-3.5" />
            <span>Done</span>
          </button>

          <button
            type="button"
            onClick={onClose}
            aria-label="Close Equalizer Studio"
            className="p-1 rounded-md text-text-muted hover:text-text-primary hover:bg-surface-card border border-transparent hover:border-border-subtle transition-all cursor-pointer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* 2. Interactive Bezier Spline Curve Canvas Visualizer (FR10) */}
      <div className="px-4 py-2 border-b border-border-subtle bg-surface-base/80">
        <div className="h-[120px] w-full rounded-lg border border-border-subtle/80 overflow-hidden relative shadow-inner">
          <canvas
            ref={canvasRef}
            className="w-full h-full block"
          />
        </div>
      </div>

      {/* 3. Fader Bank Strip (10-Band or 31-Band) (UX-DR7, EXPERIENCE.md §6.2) */}
      <div className="flex-1 px-4 py-2 flex items-center justify-between gap-1 overflow-x-auto min-h-[220px]">
        {activeIndices.map((bandIdx) => {
          const freq = ISO_31_FREQUENCIES[bandIdx];
          const gain = gains[bandIdx] ?? 0.0;
          return (
            <EqFader
              key={bandIdx}
              bandIndex={bandIdx}
              frequencyHz={freq}
              gainDb={gain}
              onChange={(newGain) => onGainChange(bandIdx, newGain)}
              isCompact={mode === "31"}
            />
          );
        })}
      </div>

      {/* 4. Bottom Hint & Telemetry Footer */}
      <div className="px-4 py-1.5 border-t border-border-subtle/60 bg-surface-card/40 flex items-center justify-between text-[10px] text-text-muted font-mono">
        <div className="flex items-center gap-1.5">
          <Activity className="w-3 h-3 text-accent-cyan" />
          <span>Detent snap +/-0.3 dB</span>
          <span>|</span>
          <span>Hold Shift for 5x fine precision</span>
          <span>|</span>
          <span>Double-click to reset (0.0 dB)</span>
        </div>
        <div className="flex items-center gap-2">
          <span>Range: -12.0 dB .. +12.0 dB</span>
          <kbd className="bg-surface-base px-1 rounded border border-border-subtle">
            Esc to close
          </kbd>
        </div>
      </div>
    </div>
  );
};
