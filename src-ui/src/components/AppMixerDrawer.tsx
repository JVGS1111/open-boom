import React, { useEffect, useRef } from "react";
import {
  Volume2,
  VolumeX,
  X,
  RefreshCw,
  SlidersHorizontal,
  Music,
  Globe,
  Radio,
  Headphones,
  Gamepad2,
  Tv,
  RotateCcw,
} from "lucide-react";

export interface ProcessAudioSession {
  process_id: number;
  name: string;
  volume: number;
  is_muted: boolean;
  is_active: boolean;
}

export interface AppMixerDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  sessions: ProcessAudioSession[];
  onVolumeChange: (processId: number, volume: number) => void;
  onToggleMute: (processId: number, currentMuted: boolean) => void;
  onResetProfile?: (processName: string, processId: number) => void;
  onRefresh?: () => void;
}

/**
 * Maps a volume level (0.0 to 1.5) to a formatted decibel readout string.
 */
export function volumeToDbString(vol: number, isMuted: boolean): string {
  if (isMuted || vol <= 0.001 || Number.isNaN(vol)) return "-inf dB";
  if (Math.abs(vol - 1.0) < 0.005) return "0.0 dB";
  if (vol < 1.0) {
    const gain = vol * vol;
    const db = 20 * Math.log10(gain);
    return `${db.toFixed(1)} dB`;
  }
  const db = 20 * Math.log10(vol);
  return `+${db.toFixed(1)} dB`;
}

/**
 * Returns a friendly process display name and appropriate icon.
 */
function getProcessMetadata(name: string): { label: string; icon: React.ReactNode } {
  const lower = name.toLowerCase();
  if (lower.includes("spotify")) {
    return { label: "Spotify", icon: <Music className="w-3.5 h-3.5 text-meter-green" /> };
  }
  if (lower.includes("discord")) {
    return { label: "Discord", icon: <Headphones className="w-3.5 h-3.5 text-accent-blue" /> };
  }
  if (lower.includes("chrome") || lower.includes("brave") || lower.includes("edge") || lower.includes("firefox")) {
    return { label: name.replace(/\.exe$/i, ""), icon: <Globe className="w-3.5 h-3.5 text-accent-cyan" /> };
  }
  if (lower.includes("steam") || lower.includes("game")) {
    return { label: name.replace(/\.exe$/i, ""), icon: <Gamepad2 className="w-3.5 h-3.5 text-accent-magenta" /> };
  }
  if (lower.includes("vlc") || lower.includes("media") || lower.includes("player")) {
    return { label: name.replace(/\.exe$/i, ""), icon: <Tv className="w-3.5 h-3.5 text-amber-400" /> };
  }
  return { label: name.replace(/\.exe$/i, ""), icon: <Radio className="w-3.5 h-3.5 text-text-secondary" /> };
}

export const AppMixerDrawer: React.FC<AppMixerDrawerProps> = ({
  isOpen,
  onClose,
  sessions,
  onVolumeChange,
  onToggleMute,
  onResetProfile,
  onRefresh,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);

  // Focus container once when drawer opens
  useEffect(() => {
    if (isOpen) {
      containerRef.current?.focus({ preventScroll: true });
    }
  }, [isOpen]);

  // Register Escape key listener
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.defaultPrevented) return;
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  return (
    <div
      ref={containerRef}
      tabIndex={-1}
      role="dialog"
      aria-label="Application Stream Mixer"
      aria-modal="true"
      aria-hidden={!isOpen}
      style={!isOpen ? { display: "none" } : undefined}
      className={`absolute inset-0 z-40 bg-surface-base/95 backdrop-blur-md border border-border-default rounded-xl shadow-2xl flex flex-col justify-between overflow-hidden transition-all duration-300 ease-out transform ${
        isOpen
          ? "translate-y-0 opacity-100 pointer-events-auto"
          : "translate-y-full opacity-0 pointer-events-none"
      }`}
    >
      {/* 1. Header Bar: Title, Active Badge, Refresh & Close */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border-subtle bg-surface-card/70 shrink-0">
        <div className="flex items-center gap-2.5">
          <div className="p-1.5 rounded-lg bg-accent-cyan/15 text-accent-cyan border border-accent-cyan/30 shadow-cyan-glow-sm">
            <SlidersHorizontal className="w-4 h-4" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h2 className="text-sm font-semibold tracking-wide text-text-primary">
                APPLICATION STREAM MIXER
              </h2>
              <span className="text-[10px] font-mono px-2 py-0.5 rounded border text-accent-cyan bg-accent-cyan/10 border-accent-cyan/30 font-medium">
                {sessions.length} {sessions.length === 1 ? "Session" : "Sessions"}
              </span>
            </div>
            <p className="text-[11px] text-text-muted mt-0.5">
              Independent process audio session faders and mute routing (UX-DR9, FR21, FR22)
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {onRefresh && (
            <button
              type="button"
              onClick={onRefresh}
              title="Refresh Audio Sessions"
              aria-label="Refresh audio sessions"
              className="p-1.5 rounded hover:bg-surface-elevated text-text-secondary hover:text-text-primary transition-colors cursor-pointer border border-transparent hover:border-border-subtle"
            >
              <RefreshCw className="w-4 h-4" />
            </button>
          )}
          <button
            type="button"
            onClick={onClose}
            title="Close Mixer (Esc)"
            aria-label="Close Mixer"
            className="p-1.5 rounded hover:bg-surface-elevated text-text-secondary hover:text-text-primary transition-colors cursor-pointer border border-transparent hover:border-border-subtle"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* 2. Session List Body */}
      <div className="flex-1 overflow-y-auto px-4 py-3 space-y-2">
        {sessions.length === 0 ? (
          <div className="h-full min-h-[220px] flex flex-col items-center justify-center text-center p-6 border border-dashed border-border-subtle rounded-xl bg-surface-card/30">
            <div className="w-12 h-12 rounded-full bg-surface-card border border-border-default flex items-center justify-center text-text-muted mb-3">
              <Radio className="w-6 h-6" />
            </div>
            <h3 className="text-sm font-medium text-text-secondary">
              No active audio streams detected
            </h3>
            <p className="text-xs text-text-muted max-w-sm mt-1">
              Start audio playback in any desktop app (media player, game, or browser) to control its volume independently.
            </p>
          </div>
        ) : (
          sessions.map((session) => {
            const { label, icon } = getProcessMetadata(session.name);
            const percentage = Math.round(session.volume * 100);
            const dbText = volumeToDbString(session.volume, session.is_muted);

            return (
              <div
                key={session.process_id}
                data-testid={`app-mixer-row-${session.process_id}`}
                className="h-[48px] px-3 rounded-lg bg-surface-card border border-border-subtle hover:border-border-default flex items-center justify-between gap-3 transition-colors select-none"
              >
                {/* App Identity: Icon, Activity Indicator, Process Name & PID */}
                <div className="flex items-center gap-2.5 min-w-[160px] max-w-[200px] shrink-0">
                  <div className="relative w-7 h-7 rounded-md bg-surface-base border border-border-default flex items-center justify-center shrink-0">
                    {icon}
                    {/* Live playback indicator dot */}
                    <span
                      title={session.is_active ? "Actively outputting audio" : "Idle stream"}
                      className={`absolute -top-0.5 -right-0.5 w-2 h-2 rounded-full ${
                        session.is_active
                          ? "bg-meter-green shadow-[0_0_6px_rgba(34,197,94,0.8)] animate-pulse"
                          : "bg-border-subtle"
                      }`}
                    />
                  </div>

                  <div className="truncate">
                    <div className="text-xs font-medium text-text-primary truncate" title={session.name}>
                      {label}
                    </div>
                    <div className="text-[10px] font-mono text-text-muted">
                      PID {session.process_id}
                    </div>
                  </div>
                </div>

                {/* Center: Volume Slider (0% to 150%) with snap-to-100% */}
                <div className="flex-1 flex items-center gap-3 max-w-[380px]">
                  <input
                    type="range"
                    min="0"
                    max="1.5"
                    step="0.01"
                    disabled={session.is_muted}
                    value={session.is_muted ? 0 : session.volume}
                    onChange={(e) =>
                      onVolumeChange(session.process_id, parseFloat(e.target.value))
                    }
                    onDoubleClick={() => onVolumeChange(session.process_id, 1.0)}
                    title={`Volume for ${session.name} (0% - 150%). Double-click to snap to 100% (0.0 dB).`}
                    aria-label={`Volume for ${label}`}
                    aria-valuemin={0}
                    aria-valuemax={1.5}
                    aria-valuenow={session.is_muted ? 0 : session.volume}
                    aria-valuetext={`${percentage}% (${dbText})`}
                    className={`flex-1 h-1.5 rounded-lg appearance-none cursor-pointer accent-accent-cyan bg-border-subtle transition-opacity ${
                      session.is_muted ? "opacity-40 cursor-not-allowed" : "opacity-100"
                    }`}
                  />

                  {/* Numeric dB & % Readout */}
                  <span
                    onDoubleClick={() => onVolumeChange(session.process_id, 1.0)}
                    title="Double-click to reset to 100% (0.0 dB)"
                    className="text-[11px] font-mono text-text-secondary w-24 text-right cursor-pointer select-none hover:text-accent-cyan transition-colors"
                  >
                    {percentage}% ({dbText})
                  </span>
                </div>

                {/* Right: Actions (Reset Custom Profile & Mute Toggle Button) */}
                <div className="flex items-center gap-1.5 shrink-0">
                  {onResetProfile && (Math.abs(session.volume - 1.0) > 0.005 || session.is_muted) && (
                    <button
                      type="button"
                      data-testid={`reset-app-profile-${session.process_id}`}
                      onClick={() => onResetProfile(session.name, session.process_id)}
                      title={`Reset custom volume preference for ${label} to 100% (0.0 dB)`}
                      aria-label={`Reset volume for ${label}`}
                      className="w-7 h-7 rounded flex items-center justify-center transition-colors cursor-pointer border bg-surface-base hover:bg-surface-elevated border-border-subtle text-text-muted hover:text-accent-cyan"
                    >
                      <RotateCcw className="w-3.5 h-3.5" />
                    </button>
                  )}

                  <button
                    type="button"
                    onClick={() => onToggleMute(session.process_id, session.is_muted)}
                    title={session.is_muted ? `Unmute ${label}` : `Mute ${label}`}
                    aria-label={session.is_muted ? `Unmute ${label}` : `Mute ${label}`}
                    className={`w-7 h-7 rounded flex items-center justify-center transition-colors cursor-pointer border ${
                      session.is_muted
                        ? "bg-meter-red/15 border-meter-red/40 text-meter-red"
                        : "bg-surface-base hover:bg-surface-elevated border-border-subtle text-text-secondary hover:text-text-primary"
                    }`}
                  >
                    {session.is_muted ? (
                      <VolumeX className="w-3.5 h-3.5" />
                    ) : (
                      <Volume2 className="w-3.5 h-3.5" />
                    )}
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* 3. Footer Bar: Keyboard Hints & System Routing Notice */}
      <div className="px-4 py-2 border-t border-border-subtle bg-surface-card/40 flex items-center justify-between text-[11px] text-text-muted shrink-0 select-none">
        <div className="flex items-center gap-3">
          <span>
            Shortcut: <kbd className="px-1.5 py-0.5 rounded bg-surface-base border border-border-subtle font-mono text-[10px]">Ctrl+Shift+M</kbd>
          </span>
          <span>•</span>
          <span>Double-click slider to reset 100% (0.0 dB)</span>
          <span>•</span>
          <span className="text-accent-cyan/80">Auto-persisted to app_volumes.json (FR23)</span>
        </div>
        <div>
          <span>Core Audio MTA Isolated • Zero Audio Duplication</span>
        </div>
      </div>
    </div>
  );
};
