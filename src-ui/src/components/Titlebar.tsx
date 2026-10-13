import { Sliders, Volume2, VolumeX, Minus, X, Settings, Layers, ArrowUpCircle } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { MasterPowerSwitch } from "./MasterPowerSwitch";
import { DeviceSelector, AudioDeviceInfo } from "./DeviceSelector";
import { LanguageMenu } from "./LanguageMenu";
import { useI18n } from "../i18n";

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
  onOpenSettings?: (initialTab?: "general" | "updates") => void;
  onToggleAppMixer?: () => void;
  isAppMixerOpen?: boolean;
  hasUpdate?: boolean;
  onOpenUpdates?: () => void;
  disabled?: boolean;
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
  onToggleAppMixer,
  isAppMixerOpen,
  hasUpdate = false,
  onOpenUpdates,
  disabled = false,
}) => {
  const { t } = useI18n();

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
      <div className="flex items-center gap-2.5 shrink-0 pointer-events-none" data-tauri-drag-region>
        <div className="w-7 h-7 rounded-lg bg-surface-card border border-border-default flex items-center justify-center text-accent-cyan shadow-cyan-glow-sm">
          <Sliders className="w-4 h-4" />
        </div>
        <div className="flex items-center gap-2">
          <span className="font-display font-bold text-sm tracking-wider text-text-primary">
            OPENBOOM
          </span>
          <span className="text-[10px] font-mono px-1.5 py-0.5 bg-accent-cyan/15 text-accent-cyan border border-accent-cyan/30 rounded font-medium">
            {t.titlebar.brandSubtitle}
          </span>
        </div>
      </div>

      {/* Center Controls: Master Power, Device, Volume */}
      <div className="flex items-center gap-3 shrink-0">
        {/* Master Power Pill Switch (48px x 26px, UX-DR2, FR27) */}
        <div
          className={`flex items-center gap-2 shrink-0 transition-opacity ${
            disabled ? "opacity-40 pointer-events-none select-none cursor-not-allowed" : ""
          }`}
          title={disabled ? t.driverOnboarding.gatingTooltip : undefined}
        >
          <MasterPowerSwitch
            enabled={masterPower}
            onToggle={onTogglePower}
          />
          <span className={`text-[11px] font-semibold tracking-wider uppercase transition-colors ${
            masterPower ? "text-accent-cyan" : "text-text-muted"
          }`}>
            {masterPower ? t.common.on : t.common.off}
          </span>
        </div>

        <div className="h-4 w-px bg-border-subtle shrink-0" />

        {/* Dynamic Output Endpoint Selector Popover (FR1, EXPERIENCE.md §4.3) */}
        <div className="shrink-0">
          <DeviceSelector
            devices={devices}
            activeDeviceId={activeDeviceId}
            onSelectDevice={onSelectDevice}
          />
        </div>

        <div className="h-4 w-px bg-border-subtle shrink-0" />

        {/* Master Volume & Boost Slider (0%–150%, UX-DR3) */}
        <div
          className={`flex items-center gap-2 shrink-0 transition-opacity ${
            disabled ? "opacity-40 pointer-events-none select-none cursor-not-allowed" : ""
          }`}
          title={disabled ? t.driverOnboarding.gatingTooltip : undefined}
        >
          <button
            type="button"
            onClick={onToggleMute}
            className="text-text-secondary hover:text-text-primary transition-colors p-1 cursor-pointer"
            title={isMuted ? t.titlebar.unmute : t.titlebar.mute}
            aria-label={isMuted ? t.titlebar.unmuteAudioAria : t.titlebar.muteAudioAria}
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
            onChange={(e) => {
              const val = parseFloat(e.target.value);
              if (!Number.isNaN(val)) {
                onVolumeChange(val);
              }
            }}
            onDoubleClick={() => onVolumeChange(1.0)} // Snap to 100% (0.0 dB)
            title={t.titlebar.masterVolumeTooltip}
            aria-label={t.titlebar.masterVolume}
            aria-valuemin={0}
            aria-valuemax={1.5}
            aria-valuenow={isMuted ? 0 : volume}
            aria-valuetext={getDbString()}
            className="w-20 lg:w-24 h-1.5 bg-border-subtle rounded-lg appearance-none cursor-pointer accent-accent-cyan shrink-0"
          />
          <span
            onDoubleClick={() => onVolumeChange(1.0)}
            title={t.titlebar.resetVolumeTooltip}
            className="text-[11px] font-mono text-text-secondary w-14 text-right cursor-pointer select-none hover:text-text-primary transition-colors shrink-0"
          >
            {getDbString()}
          </span>
        </div>
      </div>

      {/* Window Action Controls */}
      <div className="flex items-center gap-1 shrink-0">
        {/* Per-Application Stream Mixer Drawer Launcher (UX-DR9, EXPERIENCE.md §2) */}
        <button
          type="button"
          onClick={onToggleAppMixer}
          className={`w-8 h-8 rounded flex items-center justify-center transition-all cursor-pointer ${
            isAppMixerOpen
              ? "text-accent-cyan bg-accent-cyan/15 border border-accent-cyan/40 shadow-cyan-glow-sm"
              : "text-text-secondary hover:text-text-primary hover:bg-surface-card border border-transparent"
          }`}
          title={t.titlebar.appVolumesTooltip}
          aria-label={t.titlebar.appVolumesTooltip}
          aria-expanded={isAppMixerOpen}
        >
          <Layers className="w-4 h-4" />
        </button>

        {/* Dedicated Language Menu Dropdown */}
        <LanguageMenu />

        {/* Update Notification Action Badge */}
        {hasUpdate && (
          <button
            type="button"
            onClick={onOpenUpdates || (() => onOpenSettings?.("updates"))}
            className="h-7 px-2 rounded flex items-center gap-1.5 text-amber-400 bg-amber-400/15 border border-amber-400/40 hover:bg-amber-400/25 transition-all shadow-[0_0_10px_rgba(251,191,36,0.25)] animate-pulse cursor-pointer mr-0.5"
            title={t.titlebar.updateTooltip}
            aria-label={t.titlebar.updateAvailable}
          >
            <ArrowUpCircle className="w-3.5 h-3.5 text-amber-400" />
            <span className="text-[10px] font-bold tracking-wider uppercase font-mono hidden sm:inline">
              Update
            </span>
          </button>
        )}

        <button
          type="button"
          onClick={() => onOpenSettings?.("general")}
          className="w-8 h-8 rounded flex items-center justify-center text-text-secondary hover:text-text-primary hover:bg-surface-card transition-colors cursor-pointer mr-0.5"
          title={t.titlebar.settingsTooltip}
          aria-label={t.titlebar.settingsTooltip}
        >
          <Settings className="w-4 h-4" />
        </button>
        <button
          type="button"
          onClick={handleMinimize}
          className="w-8 h-8 rounded flex items-center justify-center text-text-secondary hover:text-text-primary hover:bg-surface-card transition-colors cursor-pointer"
          title={t.titlebar.minimizeTooltip}
        >
          <Minus className="w-4 h-4" />
        </button>
        <button
          type="button"
          onClick={handleClose}
          className="w-8 h-8 rounded flex items-center justify-center text-text-secondary hover:text-white hover:bg-meter-red/80 transition-colors cursor-pointer"
          title={t.titlebar.closeTooltip}
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
};

