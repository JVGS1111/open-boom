import React from "react";
import { Sliders, Volume2, VolumeX, Minus, X, Settings } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { MasterPowerSwitch } from "./MasterPowerSwitch";
import { DeviceSelector, AudioDeviceInfo } from "./DeviceSelector";

interface TitlebarProps {
  masterPower: boolean;
  onTogglePower: () => void;
  volume: number;
  onVolumeChange: (vol: number) => void;
  isMuted: boolean;
  onToggleMute: () => void;
  devices: AudioDeviceInfo[];
  activeDeviceId: string;
  onSelectDevice: (deviceId: string) => void;
  onOpenSettings?: () => void;
}

export const Titlebar: React.FC<TitlebarProps> = ({
  masterPower,
  onTogglePower,
  volume,
  onVolumeChange,
  isMuted,
  onToggleMute,
  devices,
  activeDeviceId,
  onSelectDevice,
  onOpenSettings,
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

  // Convert slider position (0.0 - 1.5) to perceptual dB display string:
  // - 0.0 to 1.0 maps to p^2 logarithmic taper (-12.0 dB at 0.5, 0.0 dB at 1.0)
  // - 1.01 to 1.5 maps to clean digital preamp gain (+2.6 dB at 1.35)
  const getDbString = () => {
    if (isMuted || volume <= 0.001 || Number.isNaN(volume)) return "-inf dB";
    if (Math.abs(volume - 1.0) < 0.005) return "0.0 dB";
    if (volume < 1.0) {
      const gain = volume * volume;
      const db = 20 * Math.log10(gain);
      return `${db.toFixed(1)} dB`;
    }
    const db = 20 * Math.log10(volume);
    return `+${db.toFixed(1)} dB`;
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

        {/* Dynamic Output Endpoint Selector Popover (FR1, EXPERIENCE.md §4.3) */}
        <DeviceSelector
          devices={devices}
          activeDeviceId={activeDeviceId}
          onSelectDevice={onSelectDevice}
        />

        <div className="h-4 w-px bg-border-subtle" />

        {/* Master Volume & Boost Slider (0%–150%, UX-DR3) */}
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={onToggleMute}
            className="text-text-secondary hover:text-text-primary transition-colors p-1 cursor-pointer"
            title={isMuted ? "Unmute" : "Mute"}
            aria-label={isMuted ? "Unmute audio" : "Mute audio"}
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
            title="Master Volume (0% - 150%). Double click to reset to 0 dB (100%)."
            aria-label="Master Volume and Boost"
            aria-valuemin={0}
            aria-valuemax={1.5}
            aria-valuenow={isMuted ? 0 : volume}
            aria-valuetext={getDbString()}
            className="w-24 h-1.5 bg-border-subtle rounded-lg appearance-none cursor-pointer accent-accent-cyan"
          />
          <span
            onDoubleClick={() => onVolumeChange(1.0)}
            title="Double-click to reset to 0.0 dB"
            className="text-[11px] font-mono text-text-secondary w-14 text-right cursor-pointer select-none hover:text-text-primary transition-colors"
          >
            {getDbString()}
          </span>
        </div>
      </div>

      {/* Window Action Controls */}
      <div className="flex items-center gap-1">
        <button
          type="button"
          onClick={onOpenSettings}
          className="w-8 h-8 rounded flex items-center justify-center text-text-secondary hover:text-text-primary hover:bg-surface-card transition-colors cursor-pointer mr-0.5"
          title="Settings & Preferences"
          aria-label="Settings"
        >
          <Settings className="w-4 h-4" />
        </button>
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

