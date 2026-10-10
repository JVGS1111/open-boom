import React from "react";
import { Sliders, Volume2, VolumeX, Minus, X, Headphones } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { MasterPowerSwitch } from "./MasterPowerSwitch";

interface TitlebarProps {
  masterPower: boolean;
  onTogglePower: () => void;
  volume: number;
  onVolumeChange: (vol: number) => void;
  isMuted: boolean;
  onToggleMute: () => void;
  activeDevice: string;
}

export const Titlebar: React.FC<TitlebarProps> = ({
  masterPower,
  onTogglePower,
  volume,
  onVolumeChange,
  isMuted,
  onToggleMute,
  activeDevice,
}) => {
  const handleMinimize = async () => {
    try {
      const win = getCurrentWindow();
      await win.minimize();
    } catch {
      // Fallback for browser preview
    }
  };

  const handleClose = async () => {
    try {
      const win = getCurrentWindow();
      // Intercepted by Tauri backend to hide to tray
      await win.close();
    } catch {
      // Fallback for browser preview
    }
  };

  // Convert linear volume (0.0 - 1.5) to dB display string
  const getDbString = () => {
    if (isMuted || volume <= 0 || Number.isNaN(volume)) return "-inf dB";
    if (Math.abs(volume - 1.0) < 0.01) return "0.0 dB";
    const db = 20 * Math.log10(volume);
    return `${db > 0 ? "+" : ""}${db.toFixed(1)} dB`;
  };

  return (
    <header
      data-tauri-drag-region
      className="h-[54px] bg-surface-base border-b border-border-subtle flex items-center justify-between px-4 select-none shrink-0"
    >
      {/* Brand Section */}
      <div className="flex items-center gap-3 pointer-events-none" data-tauri-drag-region>
        <div className="w-7 h-7 rounded-lg bg-surface-card border border-border-default flex items-center justify-center text-accent-cyan shadow-cyan-glow-sm">
          <Sliders className="w-4 h-4" />
        </div>
        <div className="flex items-center gap-2">
          <span className="font-display font-bold text-sm tracking-wider text-text-primary">
            OPENBOOM
          </span>
          <span className="text-[10px] font-mono px-1.5 py-0.5 bg-accent-cyan/15 text-accent-cyan border border-accent-cyan/30 rounded font-medium">
            PRO DSP
          </span>
        </div>
      </div>

      {/* Center Controls: Master Power, Device, Volume */}
      <div className="flex items-center gap-4">
        {/* Master Power Pill Switch (48px x 26px, UX-DR2, FR27) */}
        <div className="flex items-center gap-2">
          <MasterPowerSwitch
            enabled={masterPower}
            onToggle={onTogglePower}
          />
          <span className={`text-[11px] font-semibold tracking-wider uppercase transition-colors ${
            masterPower ? "text-accent-cyan" : "text-text-muted"
          }`}>
            {masterPower ? "ON" : "OFF"}
          </span>
        </div>

        <div className="h-4 w-px bg-border-subtle" />

        {/* Audio Output Endpoint Indicator */}
        <div
          className="flex items-center gap-2 bg-surface-card border border-border-subtle hover:border-border-default px-2.5 py-1 rounded-md text-xs text-text-secondary transition-colors cursor-pointer"
          title="Active WASAPI Output Endpoint"
        >
          <span className="w-1.5 h-1.5 rounded-full bg-status-online shadow-[0_0_6px_#22c55e]" />
          <Headphones className="w-3.5 h-3.5 text-text-muted" />
          <span className="max-w-[140px] truncate font-medium text-text-primary">
            {activeDevice}
          </span>
        </div>

        <div className="h-4 w-px bg-border-subtle" />

        {/* Master Volume & Boost Slider */}
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={onToggleMute}
            className="text-text-secondary hover:text-text-primary transition-colors p-1"
            title={isMuted ? "Unmute" : "Mute"}
          >
            {isMuted ? (
              <VolumeX className="w-4 h-4 text-meter-red" />
            ) : (
              <Volume2 className="w-4 h-4 text-accent-cyan" />
            )}
          </button>
          <input
            type="range"
            min="0"
            max="1.5"
            step="0.01"
            value={isMuted ? 0 : volume}
            onChange={(e) => onVolumeChange(parseFloat(e.target.value))}
            onDoubleClick={() => onVolumeChange(1.0)} // Snap to 100% (0.0 dB)
            title="Master Volume (0% - 150%). Double click to reset to 0 dB."
            className="w-24 h-1.5 bg-border-subtle rounded-lg appearance-none cursor-pointer accent-accent-cyan"
          />
          <span className="text-[11px] font-mono text-text-secondary w-14 text-right">
            {getDbString()}
          </span>
        </div>
      </div>

      {/* Window Action Controls */}
      <div className="flex items-center gap-1">
        <button
          type="button"
          onClick={handleMinimize}
          className="w-8 h-8 rounded flex items-center justify-center text-text-secondary hover:text-text-primary hover:bg-surface-card transition-colors cursor-pointer"
          title="Minimize"
        >
          <Minus className="w-4 h-4" />
        </button>
        <button
          type="button"
          onClick={handleClose}
          className="w-8 h-8 rounded flex items-center justify-center text-text-secondary hover:text-white hover:bg-meter-red/80 transition-colors cursor-pointer"
          title="Minimize to Notification Tray"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};
