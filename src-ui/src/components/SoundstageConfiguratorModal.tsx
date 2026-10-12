import React, { useEffect, useState, useRef } from "react";
import {
  X,
  RotateCcw,
  Volume2,
  VolumeX,
  Compass,
  Check,
  Orbit,
  ArrowRight,
  ArrowLeft,
  Repeat,
} from "lucide-react";
import { useI18n } from "../i18n";

export type OrbitDirection = "Clockwise" | "CounterClockwise" | "PingPong";

export interface ChannelSpec {
  index: number;
  key: string;
  name: string;
  defaultAzimuth: number;
  isLfe?: boolean;
}

export const CHANNELS: ChannelSpec[] = [
  { index: 0, key: "FL", name: "Front Left", defaultAzimuth: -30 },
  { index: 1, key: "FR", name: "Front Right", defaultAzimuth: 30 },
  { index: 2, key: "C", name: "Center", defaultAzimuth: 0 },
  { index: 3, key: "LFE", name: "Subwoofer", defaultAzimuth: 180, isLfe: true },
  { index: 4, key: "RL", name: "Rear Left", defaultAzimuth: -135 },
  { index: 5, key: "RR", name: "Rear Right", defaultAzimuth: 135 },
  { index: 6, key: "SL", name: "Surround Left", defaultAzimuth: -90 },
  { index: 7, key: "SR", name: "Surround Right", defaultAzimuth: 90 },
];

interface SoundstageConfiguratorModalProps {
  isOpen: boolean;
  onClose: () => void;
  speakerMuted: boolean[];
  speakerGainsDb: number[];
  speakerAzimuthsDeg?: number[];
  onToggleMute: (channelIndex: number) => void;
  onGainChange: (channelIndex: number, gainDb: number) => void;
  onAzimuthChange?: (channelIndex: number, azimuthDeg: number) => void;
  onResetPositions?: () => void;
  onResetAll: () => void;
  // 8D Orbital Audio Parameters (Story 6.3, CAP-2, CAP-3, CAP-4)
  orbitEnabled?: boolean;
  orbitDirection?: OrbitDirection;
  orbitPeriodSec?: number;
  orbitDepth?: number;
  onToggleOrbit?: (enabled?: boolean) => void;
  onOrbitDirectionChange?: (direction: OrbitDirection) => void;
  onOrbitPeriodChange?: (periodSec: number) => void;
  onOrbitDepthChange?: (depth: number) => void;
}

export const SoundstageConfiguratorModal: React.FC<SoundstageConfiguratorModalProps> = ({
  isOpen,
  onClose,
  speakerMuted,
  speakerGainsDb,
  speakerAzimuthsDeg,
  onToggleMute,
  onGainChange,
  onAzimuthChange,
  onResetPositions,
  onResetAll,
  orbitEnabled = false,
  orbitDirection = "Clockwise",
  orbitPeriodSec = 12.0,
  orbitDepth = 0.85,
  onToggleOrbit,
  onOrbitDirectionChange,
  onOrbitPeriodChange,
  onOrbitDepthChange,
}) => {
  const { t } = useI18n();
  const [selectedChannel, setSelectedChannel] = useState<number>(0);
  const [draggingChannel, setDraggingChannel] = useState<number | null>(null);
  const dragTravelRef = useRef<number>(0);

  const getChannelName = (key: string, defaultName: string) => {
    switch (key) {
      case "FL": return t.soundstage.channels.l;
      case "FR": return t.soundstage.channels.r;
      case "C": return t.soundstage.channels.c;
      case "LFE": return t.soundstage.channels.lfe;
      case "RL": return t.soundstage.channels.lsr;
      case "RR": return t.soundstage.channels.rsr;
      case "SL": return t.soundstage.channels.ls;
      case "SR": return t.soundstage.channels.rs;
      default: return defaultName;
    }
  };

  // Close on Escape key press
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };
    if (isOpen) {
      window.addEventListener("keydown", handleKeyDown);
    }
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  // Soundstage Polar Canvas Geometry
  const canvasSize = 320;
  const cx = 160;
  const cy = 160;
  const orbitRadius = 115;
  const lfeRadius = 65;

  const getSpeakerCoords = (ch: ChannelSpec) => {
    const azimuth =
      speakerAzimuthsDeg && speakerAzimuthsDeg[ch.index] !== undefined
        ? speakerAzimuthsDeg[ch.index]
        : ch.defaultAzimuth;
    const r = ch.isLfe ? lfeRadius : orbitRadius;

    // Convert azimuth (0° North, + clockwise, - counter-clockwise) to Cartesian SVG:
    const rad = (azimuth * Math.PI) / 180.0;
    return {
      x: cx + r * Math.sin(rad),
      y: cy - r * Math.cos(rad),
      azimuth,
    };
  };

  const handleSoundstagePointerMove = (e: React.PointerEvent<SVGSVGElement>) => {
    if (draggingChannel === null || !onAzimuthChange) return;

    dragTravelRef.current += Math.hypot(e.movementX, e.movementY);

    const rect = e.currentTarget.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const dx = mouseX - cx;
    const dy = mouseY - cy;

    // Angle from North (dy negative is 0°):
    let deg = (Math.atan2(dx, -dy) * 180.0) / Math.PI;
    if (deg > 180) deg -= 360;
    if (deg < -180) deg += 360;

    onAzimuthChange(draggingChannel, Math.round(deg));
  };

  const handlePointerUp = () => {
    setDraggingChannel(null);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-md animate-fade-in"
      onClick={onClose}
    >
      {/* Modal Dialog Card */}
      <div
        className="w-[720px] max-h-[92vh] bg-surface-elevated border border-border-default rounded-2xl shadow-2xl flex flex-col overflow-hidden text-text-primary"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border-subtle bg-surface-card/60">
          <div className="flex items-center gap-2.5">
            <div className="p-1.5 rounded-lg bg-accent-cyan/15 text-accent-cyan border border-accent-cyan/30">
              <Compass className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-semibold tracking-wide">
                {t.soundstage.calibrateModalTitle}
              </h2>
              <p className="text-[12px] text-text-muted">
                {t.soundstage.calibrateModalSubtitle}
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-surface-card border border-transparent hover:border-border-subtle transition-all cursor-pointer"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content Body: Split View (Polar Stage on Left, Channel Controls on Right) */}
        <div className="p-5 grid grid-cols-12 gap-5 overflow-y-auto">
          {/* Left Column: Top-Down Polar Soundstage SVG */}
          <div className="col-span-6 flex flex-col items-center justify-center p-3 bg-surface-base/80 border border-border-subtle rounded-xl relative">
            <span className="text-[11px] font-mono text-text-muted mb-1 flex items-center gap-1.5">
              {draggingChannel !== null ? (
                <span className="text-accent-cyan font-semibold flex items-center gap-1">
                  <Compass className="w-3 h-3 animate-spin" />
                  <span>
                    ORBIT {getChannelName(CHANNELS[draggingChannel].key, CHANNELS[draggingChannel].name).toUpperCase()}:{" "}
                    {(() => {
                      const az =
                        speakerAzimuthsDeg && speakerAzimuthsDeg[draggingChannel] !== undefined
                          ? speakerAzimuthsDeg[draggingChannel]
                          : CHANNELS[draggingChannel].defaultAzimuth;
                      return az > 0 ? `+${az}°` : `${az}°`;
                    })()}{" "}
                    AZIMUTH
                  </span>
                </span>
              ) : (
                <span>{t.soundstage.listener} PERSPECTIVE (TOP-DOWN)</span>
              )}
            </span>

            <svg
              className="w-[320px] h-[320px] select-none cursor-crosshair"
              viewBox={`0 0 ${canvasSize} ${canvasSize}`}
              onPointerMove={handleSoundstagePointerMove}
              onPointerUp={handlePointerUp}
            >
              {/* Polar Guide Concentric Rings */}
              <circle
                cx={cx}
                cy={cy}
                r="40"
                fill="none"
                stroke="#22252e"
                strokeWidth="1"
                strokeDasharray="4 4"
              />
              <circle
                cx={cx}
                cy={cy}
                r={lfeRadius}
                fill="none"
                stroke="#262933"
                strokeWidth="1"
                strokeDasharray="4 4"
              />
              <circle
                cx={cx}
                cy={cy}
                r={orbitRadius}
                fill="none"
                stroke="#2e323e"
                strokeWidth="1.5"
              />

              {/* Crosshair Axes */}
              <line
                x1={cx}
                y1="25"
                x2={cx}
                y2="295"
                stroke="#22252e"
                strokeWidth="1"
              />
              <line
                x1="25"
                y1={cy}
                x2="295"
                y2={cy}
                stroke="#22252e"
                strokeWidth="1"
              />

              {/* Central Listener Head */}
              <g transform={`translate(${cx}, ${cy})`}>
                <circle
                  r="18"
                  fill="#1a1c23"
                  stroke="#3b82f6"
                  strokeWidth="2"
                  className="drop-shadow-[0_0_6px_rgba(59,130,246,0.3)]"
                />
                {/* Nose / Front Indicator */}
                <polygon points="0,-22 -5,-16 5,-16" fill="#38bdf8" />
                {/* Headphone Earcups */}
                <rect x="-21" y="-8" width="5" height="16" rx="2" fill="#38bdf8" />
                <rect x="16" y="-8" width="5" height="16" rx="2" fill="#38bdf8" />
              </g>

              {/* 8 Virtual Speaker Nodes */}
              {CHANNELS.map((ch) => {
                const { x, y } = getSpeakerCoords(ch);
                const isMuted = speakerMuted[ch.index];
                const isSelected = selectedChannel === ch.index;
                const gain = speakerGainsDb[ch.index] || 0.0;

                const isDraggingThis = draggingChannel === ch.index;
                const azimuth =
                  speakerAzimuthsDeg && speakerAzimuthsDeg[ch.index] !== undefined
                    ? speakerAzimuthsDeg[ch.index]
                    : ch.defaultAzimuth;

                return (
                  <g
                    key={ch.key}
                    transform={`translate(${x}, ${y})`}
                    className="cursor-pointer transition-all"
                    onClick={() => {
                      if (dragTravelRef.current > 4) return;
                      setSelectedChannel(ch.index);
                      onToggleMute(ch.index);
                    }}
                    onPointerDown={(e) => {
                      dragTravelRef.current = 0;
                      setSelectedChannel(ch.index);
                      if (onAzimuthChange && !ch.isLfe) {
                        setDraggingChannel(ch.index);
                        try {
                          (e.currentTarget as Element).setPointerCapture(e.pointerId);
                        } catch {}
                      }
                    }}
                    onPointerUp={(e) => {
                      if (draggingChannel !== null) {
                        try {
                          (e.currentTarget as Element).releasePointerCapture(e.pointerId);
                        } catch {}
                        setDraggingChannel(null);
                      }
                    }}
                  >
                    {/* Live Azimuth Angle Tag when Dragging or Selected */}
                    {(isDraggingThis || (isSelected && !ch.isLfe)) && (
                      <g transform="translate(0, -22)">
                        <rect
                          x="-18"
                          y="-7"
                          width="36"
                          height="14"
                          rx="3"
                          fill="#0b0d13"
                          stroke="#38bdf8"
                          strokeWidth="1"
                        />
                        <text
                          y="3"
                          textAnchor="middle"
                          className="text-[8px] font-mono fill-accent-cyan font-bold"
                        >
                          {azimuth > 0 ? `+${azimuth}°` : `${azimuth}°`}
                        </text>
                      </g>
                    )}

                    {/* Speaker Outer Disc */}
                    <circle
                      r="16"
                      fill={isMuted ? "#1a1618" : isSelected ? "#1c2638" : "#1e222b"}
                      stroke={
                        isMuted
                          ? "#ef4444"
                          : isSelected
                          ? "#38bdf8"
                          : "#3b82f6"
                      }
                      strokeWidth={isSelected ? "2.5" : "1.5"}
                      className={`transition-all duration-150 ${
                        isMuted
                          ? "drop-shadow-[0_0_4px_rgba(239,68,68,0.5)]"
                          : "drop-shadow-[0_0_8px_rgba(56,189,248,0.35)]"
                      }`}
                    />

                    {/* Muted Strike-Through Red Line */}
                    {isMuted && (
                      <line
                        x1="-12"
                        y1="12"
                        x2="12"
                        y2="-12"
                        stroke="#ef4444"
                        strokeWidth="2.5"
                        strokeLinecap="round"
                      />
                    )}

                    {/* Speaker Text Tag */}
                    <text
                      y="3.5"
                      textAnchor="middle"
                      className={`text-[9px] font-mono font-bold select-none ${
                        isMuted
                          ? "fill-red-400"
                          : isSelected
                          ? "fill-accent-cyan"
                          : "fill-text-primary"
                      }`}
                    >
                      {ch.key}
                    </text>

                    {/* Gain Trim Pill Tag */}
                    {!isMuted && Math.abs(gain) > 0.05 && (
                      <g transform="translate(0, 22)">
                        <rect
                          x="-14"
                          y="-6"
                          width="28"
                          height="12"
                          rx="3"
                          fill="#0f1117"
                          stroke="#38bdf8"
                          strokeWidth="0.8"
                        />
                        <text
                          y="2.5"
                          textAnchor="middle"
                          className="text-[7.5px] font-mono fill-accent-cyan font-bold"
                        >
                          {gain > 0 ? `+${gain.toFixed(1)}` : gain.toFixed(1)}
                        </text>
                      </g>
                    )}
                  </g>
                );
              })}
            </svg>

            <div className="flex items-center justify-between w-full mt-2 text-[11px] text-text-muted font-mono px-2">
              <span>Click node to Mute / Unmute</span>
              <span>Drag node to rotate</span>
            </div>
          </div>

          {/* Right Column: Channel Trim Faders & Status */}
          <div className="col-span-6 flex flex-col justify-between">
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-[12px] font-semibold text-text-secondary tracking-wider uppercase">
                  Channel Trim & Mute Rack
                </span>
                <span className="text-[11px] font-mono text-text-muted">
                  Range: -6.0 dB .. +6.0 dB
                </span>
              </div>

              {/* Channel Faders List */}
              <div className="flex flex-col gap-1.5 max-h-[300px] overflow-y-auto pr-1">
                {CHANNELS.map((ch) => {
                  const isMuted = speakerMuted[ch.index];
                  const gain = speakerGainsDb[ch.index] || 0.0;
                  const isSelected = selectedChannel === ch.index;

                  return (
                    <div
                      key={ch.key}
                      onClick={() => setSelectedChannel(ch.index)}
                      className={`flex items-center justify-between p-2 rounded-lg border transition-all cursor-pointer ${
                        isSelected
                          ? "bg-surface-card border-accent-cyan/40 shadow-sm"
                          : "bg-surface-card/40 border-border-subtle hover:border-border-default"
                      }`}
                    >
                      {/* Channel Badge, Name & Azimuth */}
                      <div className="flex items-center gap-2 w-32">
                        <span
                          className={`w-7 h-5 flex items-center justify-center text-[10px] font-mono font-bold rounded ${
                            isMuted
                              ? "bg-red-950/60 text-red-400 border border-red-500/30"
                              : "bg-surface-elevated text-accent-cyan border border-accent-cyan/30"
                          }`}
                        >
                          {ch.key}
                        </span>
                        <div className="flex flex-col truncate">
                          <span
                            className={`text-[12px] truncate font-medium ${
                              isMuted ? "text-text-muted line-through" : "text-text-primary"
                            }`}
                          >
                            {getChannelName(ch.key, ch.name)}
                          </span>
                          {!ch.isLfe && (
                            <span className="text-[9.5px] font-mono text-text-muted">
                              {(() => {
                                const az =
                                  speakerAzimuthsDeg && speakerAzimuthsDeg[ch.index] !== undefined
                                    ? speakerAzimuthsDeg[ch.index]
                                    : ch.defaultAzimuth;
                                return az > 0 ? `+${az}°` : `${az}°`;
                              })()}
                            </span>
                          )}
                        </div>
                      </div>

                      {/* Trim Fader Slider */}
                      <div className="flex items-center gap-2 flex-1 mx-3">
                        <input
                          type="range"
                          min="-6.0"
                          max="6.0"
                          step="0.1"
                          value={gain}
                          disabled={isMuted}
                          onDoubleClick={() => onGainChange(ch.index, 0.0)}
                          onChange={(e) =>
                            onGainChange(ch.index, parseFloat(e.target.value))
                          }
                          className="w-full h-1 bg-border-subtle rounded appearance-none cursor-pointer accent-accent-cyan disabled:opacity-30"
                        />
                        <span
                          className={`w-12 text-right text-[11px] font-mono font-bold ${
                            isMuted
                              ? "text-text-muted"
                              : gain > 0
                              ? "text-accent-cyan"
                              : gain < 0
                              ? "text-text-muted"
                              : "text-text-secondary"
                          }`}
                        >
                          {gain > 0 ? `+${gain.toFixed(1)}` : gain.toFixed(1)} dB
                        </span>
                      </div>

                      {/* Mute Toggle Button */}
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          onToggleMute(ch.index);
                        }}
                        title={isMuted ? "Unmute Channel" : "Mute Channel"}
                        className={`p-1.5 rounded border transition-colors cursor-pointer ${
                          isMuted
                            ? "bg-red-500/20 text-red-400 border-red-500/40 hover:bg-red-500/30"
                            : "bg-surface-elevated text-text-muted hover:text-text-primary border-border-subtle hover:border-border-default"
                        }`}
                      >
                        {isMuted ? (
                          <VolumeX className="w-3.5 h-3.5" />
                        ) : (
                          <Volume2 className="w-3.5 h-3.5" />
                        )}
                      </button>
                    </div>
                  );
                })}
              </div>
            </div>

            {/* 8D Orbital Audio Parameters Panel (Story 6.3, CAP-2, CAP-3, CAP-4, CAP-5) */}
            <div className="mt-4 p-3.5 bg-surface-base/90 border border-border-subtle rounded-xl flex flex-col gap-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className={`p-1 rounded-md border transition-all ${
                    orbitEnabled
                      ? "bg-accent-cyan/20 text-accent-cyan border-accent-cyan/40 shadow-[0_0_8px_rgba(56,189,248,0.3)]"
                      : "bg-surface-elevated text-text-muted border-border-subtle"
                  }`}>
                    <Orbit className={`w-3.5 h-3.5 ${orbitEnabled ? "animate-spin" : ""}`} style={{ animationDuration: "6s" }} />
                  </div>
                  <div>
                    <h3 className="text-[11.5px] font-semibold text-text-primary uppercase tracking-wider">
                      {t.soundstage.orbitSectionTitle}
                    </h3>
                    <p className="text-[10px] text-text-muted">
                      {t.soundstage.orbitSectionSubtitle}
                    </p>
                  </div>
                </div>

                {/* 8D Orbit Master Toggle */}
                {onToggleOrbit && (
                  <button
                    type="button"
                    onClick={() => onToggleOrbit(!orbitEnabled)}
                    aria-label={t.soundstage.orbitToggleAria}
                    aria-pressed={orbitEnabled}
                    className={`px-2.5 py-1 text-[11px] font-semibold rounded-md border transition-all flex items-center gap-1.5 cursor-pointer ${
                      orbitEnabled
                        ? "bg-accent-cyan/20 border-accent-cyan text-accent-cyan shadow-[0_0_10px_rgba(56,189,248,0.35)]"
                        : "bg-surface-card border-border-subtle text-text-secondary hover:text-text-primary hover:border-border-default"
                    }`}
                  >
                    <span className={`w-1.5 h-1.5 rounded-full ${orbitEnabled ? "bg-accent-cyan shadow-[0_0_6px_#38bdf8]" : "bg-text-muted"}`} />
                    <span>{t.soundstage.orbit8D} {orbitEnabled ? t.common.on : t.common.off}</span>
                  </button>
                )}
              </div>

              {/* 3-Way Directional Segmented Selector */}
              <div className="flex flex-col gap-1.5">
                <span className="text-[10.5px] font-medium text-text-secondary">
                  {t.soundstage.orbitDirectionLabel}
                </span>
                <div
                  role="radiogroup"
                  aria-label={t.soundstage.orbitDirectionLabel}
                  className="grid grid-cols-3 gap-1.5 bg-surface-elevated/70 p-1 rounded-lg border border-border-subtle"
                >
                  {/* Clockwise */}
                  <button
                    type="button"
                    role="radio"
                    aria-checked={orbitDirection === "Clockwise"}
                    aria-label={t.soundstage.orbitClockwise}
                    onClick={() => onOrbitDirectionChange?.("Clockwise")}
                    className={`flex items-center justify-center gap-1.5 py-1.5 px-2 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
                      orbitDirection === "Clockwise"
                        ? "bg-accent-cyan/25 border border-accent-cyan text-accent-cyan shadow-[0_0_8px_rgba(56,189,248,0.35)] font-semibold"
                        : "border border-transparent text-text-secondary hover:text-text-primary hover:bg-surface-card"
                    }`}
                  >
                    <ArrowRight className="w-3 h-3" />
                    <span>{t.soundstage.orbitClockwise}</span>
                  </button>

                  {/* Counter-Clockwise */}
                  <button
                    type="button"
                    role="radio"
                    aria-checked={orbitDirection === "CounterClockwise"}
                    aria-label={t.soundstage.orbitCounterClockwise}
                    onClick={() => onOrbitDirectionChange?.("CounterClockwise")}
                    className={`flex items-center justify-center gap-1.5 py-1.5 px-2 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
                      orbitDirection === "CounterClockwise"
                        ? "bg-accent-cyan/25 border border-accent-cyan text-accent-cyan shadow-[0_0_8px_rgba(56,189,248,0.35)] font-semibold"
                        : "border border-transparent text-text-secondary hover:text-text-primary hover:bg-surface-card"
                    }`}
                  >
                    <ArrowLeft className="w-3 h-3" />
                    <span>{t.soundstage.orbitCounterClockwise}</span>
                  </button>

                  {/* Ping-Pong */}
                  <button
                    type="button"
                    role="radio"
                    aria-checked={orbitDirection === "PingPong"}
                    aria-label={t.soundstage.orbitPingPong}
                    onClick={() => onOrbitDirectionChange?.("PingPong")}
                    className={`flex items-center justify-center gap-1.5 py-1.5 px-2 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
                      orbitDirection === "PingPong"
                        ? "bg-accent-cyan/25 border border-accent-cyan text-accent-cyan shadow-[0_0_8px_rgba(56,189,248,0.35)] font-semibold"
                        : "border border-transparent text-text-secondary hover:text-text-primary hover:bg-surface-card"
                    }`}
                  >
                    <Repeat className="w-3 h-3" />
                    <span>{t.soundstage.orbitPingPong}</span>
                  </button>
                </div>
              </div>

              {/* Sliders: Period (Speed) & Depth */}
              <div className="grid grid-cols-2 gap-4">
                {/* Revolution Period Slider */}
                <div className="flex flex-col gap-1">
                  <div className="flex items-center justify-between">
                    <span className="text-[10.5px] font-medium text-text-secondary">
                      {t.soundstage.orbitPeriodLabel}
                    </span>
                    <span
                      title={t.soundstage.orbitPeriodResetTooltip}
                      onDoubleClick={() => onOrbitPeriodChange?.(12.0)}
                      className="text-[11px] font-mono font-bold text-accent-cyan cursor-pointer hover:underline"
                    >
                      {orbitPeriodSec.toFixed(1)}s
                    </span>
                  </div>
                  <input
                    type="range"
                    min="2.0"
                    max="30.0"
                    step="0.5"
                    aria-label={t.soundstage.orbitPeriodLabel}
                    aria-valuetext={`${orbitPeriodSec.toFixed(1)}s`}
                    value={orbitPeriodSec}
                    title={t.soundstage.orbitPeriodResetTooltip}
                    onDoubleClick={() => onOrbitPeriodChange?.(12.0)}
                    onChange={(e) => {
                      const val = parseFloat(e.target.value);
                      const safeVal = isNaN(val) ? 12.0 : Math.min(30.0, Math.max(2.0, val));
                      onOrbitPeriodChange?.(safeVal);
                    }}
                    className="w-full h-1 bg-border-subtle rounded appearance-none cursor-pointer accent-accent-cyan"
                  />
                  <div className="flex justify-between text-[9px] font-mono text-text-muted">
                    <span>2.0s</span>
                    <span>12.0s</span>
                    <span>30.0s</span>
                  </div>
                </div>

                {/* Orbital Depth Slider */}
                <div className="flex flex-col gap-1">
                  <div className="flex items-center justify-between">
                    <span className="text-[10.5px] font-medium text-text-secondary">
                      {t.soundstage.orbitDepthLabel}
                    </span>
                    <span
                      title={t.soundstage.orbitDepthResetTooltip}
                      onDoubleClick={() => onOrbitDepthChange?.(0.85)}
                      className="text-[11px] font-mono font-bold text-accent-cyan cursor-pointer hover:underline"
                    >
                      {Math.round(orbitDepth * 100)}%
                    </span>
                  </div>
                  <input
                    type="range"
                    min="0"
                    max="100"
                    step="1"
                    aria-label={t.soundstage.orbitDepthLabel}
                    aria-valuetext={`${Math.round(orbitDepth * 100)}%`}
                    value={Math.round(orbitDepth * 100)}
                    title={t.soundstage.orbitDepthResetTooltip}
                    onDoubleClick={() => onOrbitDepthChange?.(0.85)}
                    onChange={(e) => {
                      const val = parseInt(e.target.value, 10);
                      const safeVal = isNaN(val) ? 0.85 : Math.min(1.0, Math.max(0.0, val / 100));
                      onOrbitDepthChange?.(safeVal);
                    }}
                    className="w-full h-1 bg-border-subtle rounded appearance-none cursor-pointer accent-accent-cyan"
                  />
                  <div className="flex justify-between text-[9px] font-mono text-text-muted">
                    <span>0%</span>
                    <span>85%</span>
                    <span>100%</span>
                  </div>
                </div>
              </div>
            </div>

            {/* Quick Action Buttons */}
            <div className="flex items-center justify-between pt-3 border-t border-border-subtle mt-3">
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={onResetAll}
                  className="flex items-center gap-1.5 px-3 py-1.5 text-[11px] font-medium bg-surface-card border border-border-subtle hover:border-border-default rounded-md text-text-secondary hover:text-text-primary transition-all cursor-pointer"
                >
                  <RotateCcw className="w-3.5 h-3.5" />
                  <span>{t.soundstage.resetTrims}</span>
                </button>

                {onResetPositions && (
                  <button
                    type="button"
                    onClick={onResetPositions}
                    className="flex items-center gap-1.5 px-3 py-1.5 text-[11px] font-medium bg-surface-card border border-border-subtle hover:border-border-default rounded-md text-text-secondary hover:text-text-primary transition-all cursor-pointer"
                  >
                    <Compass className="w-3.5 h-3.5 text-accent-cyan" />
                    <span>{t.soundstage.resetSpeakers}</span>
                  </button>
                )}
              </div>

              <button
                type="button"
                onClick={onClose}
                className="flex items-center gap-1.5 px-5 py-1.5 text-[12px] font-semibold bg-accent-cyan text-background hover:bg-accent-cyan/90 rounded-md transition-all shadow-cyan-glow-sm cursor-pointer"
              >
                <Check className="w-4 h-4" />
                <span>{t.common.done}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
