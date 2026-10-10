import React, { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Titlebar } from "./components/Titlebar";
import { SoundstageDial } from "./components/SoundstageDial";
import { EqualizerPreview } from "./components/EqualizerPreview";
import { EffectsRackPreview, EffectModule } from "./components/EffectsRackPreview";
import { StatusBar } from "./components/StatusBar";
import { NotificationBanner } from "./components/NotificationBanner";
import { AudioDeviceInfo } from "./components/DeviceSelector";
import { SettingsModal } from "./components/SettingsModal";
import { SoundstageConfiguratorModal } from "./components/SoundstageConfiguratorModal";

export const App: React.FC = () => {
  // Master Engine State
  const [masterPower, setMasterPower] = useState(true);
  const [volume, setVolume] = useState(0.8);
  const [isMuted, setIsMuted] = useState(false);

  // Audio Endpoints & Routing State
  const [devices, setDevices] = useState<AudioDeviceInfo[]>([
    {
      id: "default",
      name: "Headphones (Realtek Audio)",
      is_default: true,
    },
  ]);
  const [activeDeviceId, setActiveDeviceId] = useState("default");
  const [notification, setNotification] = useState<string | null>(null);

  // 3D Spatializer State
  const [surroundEnabled, setSurroundEnabled] = useState(true);
  const [surroundIntensity, setSurroundIntensity] = useState(0.72);

  // Equalizer State (31 Bands)
  const [activeEqPreset, setActiveEqPreset] = useState("Harman Acoustic");
  const [eqGains] = useState<number[]>([
    2.5, 3.0, 2.0, 1.5, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0,
    1.5, 1.0, 0.5, 0.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 2.0, 1.5, 1.0, 0.5,
    0.0, -0.5,
  ]);
  const [preampOffsetDb] = useState(-3.0);

  // Sweeteners Rack State
  const [effectModules, setEffectModules] = useState<EffectModule[]>([
    {
      id: 0,
      name: "Ambience",
      description: "FDN Reverb Matrix",
      enabled: true,
      amount: 0.35,
    },
    {
      id: 1,
      name: "Fidelity",
      description: "Harmonic Clarity Exciter",
      enabled: true,
      amount: 0.45,
    },
    {
      id: 2,
      name: "Night Mode",
      description: "Dynamic Range Soft-Knee",
      enabled: false,
      amount: 0.5,
    },
    {
      id: 3,
      name: "Spatial Widener",
      description: "Mid/Side Stereo Crossfeed",
      enabled: true,
      amount: 0.6,
    },
  ]);

  // Telemetry & A/B Bypass State
  const [bypassActive, setBypassActive] = useState(false);
  const [latencyMs, setLatencyMs] = useState(11.4);
  const [limiterGainReductionDb, setLimiterGainReductionDb] = useState(0.0);
  const [limiterHeadroomDb, setLimiterHeadroomDb] = useState(20.0);
  const [limiterActive, setLimiterActive] = useState(false);
  const [sampleRate, setSampleRate] = useState(48000);
  const [bitDepth, setBitDepth] = useState(32);
  const [usingVirtualBridge, setUsingVirtualBridge] = useState(false);

  // Settings Modal & Windows Startup State (Story 1.7)
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [launchOnStartup, setLaunchOnStartup] = useState(false);
  const [hasLoadedConfig, setHasLoadedConfig] = useState(false);
  const saveTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Soundstage Configurator Modal State (Story 2.3 & 2.4, FR7)
  const [isConfiguratorOpen, setIsConfiguratorOpen] = useState(false);
  const [speakerMuted, setSpeakerMuted] = useState<boolean[]>([
    false, false, false, false, false, false, false, false,
  ]);
  const [speakerGainsDb, setSpeakerGainsDb] = useState<number[]>([
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
  ]);
  const [speakerAzimuthsDeg, setSpeakerAzimuthsDeg] = useState<number[]>([
    -30, 30, 0, 180, -135, 135, -90, 90,
  ]);


  // Helper to dispatch IPC command safely to Tauri
  const dispatchCommand = async (commandObj: unknown) => {
    try {
      await invoke("dispatch_command", { cmd: commandObj });
    } catch {
      // Fallback for browser preview
    }
  };

  // 1. Restore configuration and Windows startup settings on mount (FR29)
  useEffect(() => {
    const restoreConfig = async () => {
      try {
        const config = await invoke<{
          master_power: boolean;
          master_volume: number;
          master_boost: number;
          is_muted: boolean;
          active_output_device: string | null;
          window_width: number;
          window_height: number;
          window_x: number | null;
          window_y: number | null;
          graphic_eq_gains_db: number[];
          surround_enabled: boolean;
          surround_intensity: number;
          speaker_muted?: boolean[];
          speaker_gains_db?: number[];
          speaker_azimuths_deg?: number[];
          bypass_active: boolean;
          launch_on_startup: boolean;
        }>("load_app_config");

        if (config) {
          setMasterPower(config.master_power);
          dispatchCommand({ SetMasterPower: config.master_power });

          // Reconstruct slider from stored volume and boost
          const restoredVol =
            config.master_boost > 1.0
              ? config.master_boost
              : Math.sqrt(config.master_volume);
          setVolume(restoredVol);
          dispatchCommand({
            SetMasterVolume: {
              volume: config.master_volume,
              boost: config.master_boost,
            },
          });

          setIsMuted(config.is_muted);
          dispatchCommand({ SetMute: config.is_muted });

          if (config.active_output_device) {
            setActiveDeviceId(config.active_output_device);
            try {
              await invoke("set_audio_device", {
                deviceId: config.active_output_device,
              });
            } catch {
              // Graceful fallback
            }
          }

          setSurroundEnabled(config.surround_enabled);
          setSurroundIntensity(config.surround_intensity);
          dispatchCommand({
            SetSurround3D: {
              enabled: config.surround_enabled,
              intensity: config.surround_intensity,
            },
          });

          if (config.speaker_muted && config.speaker_muted.length === 8) {
            setSpeakerMuted(config.speaker_muted);
            config.speaker_muted.forEach((m, idx) => {
              if (m) dispatchCommand({ SetSpeakerMute: { channel: idx, muted: true } });
            });
          }

          if (config.speaker_gains_db && config.speaker_gains_db.length === 8) {
            setSpeakerGainsDb(config.speaker_gains_db);
            config.speaker_gains_db.forEach((g, idx) => {
              if (g !== 0.0) dispatchCommand({ SetSpeakerGain: { channel: idx, gain_db: g } });
            });
          }

          if (config.speaker_azimuths_deg && config.speaker_azimuths_deg.length === 8) {
            setSpeakerAzimuthsDeg(config.speaker_azimuths_deg);
            config.speaker_azimuths_deg.forEach((az, idx) => {
              dispatchCommand({ SetSpeakerAzimuth: { channel: idx, azimuth_deg: az } });
            });
          }

          setBypassActive(config.bypass_active);
          dispatchCommand({ SetBypass: config.bypass_active });

          setLaunchOnStartup(config.launch_on_startup);
        }
      } catch {
        // Browser mode fallback
      }

      // Check current Windows registry status
      try {
        const startupActive = await invoke<boolean>("get_launch_on_startup");
        setLaunchOnStartup(startupActive);
      } catch {
        // Browser mode fallback
      }

      setHasLoadedConfig(true);
    };

    restoreConfig();
  }, []);

  // 2. Enumerate active audio render endpoints on mount
  useEffect(() => {
    const fetchDevices = async () => {
      try {
        const enumerated = await invoke<AudioDeviceInfo[]>("get_audio_devices");
        if (enumerated && enumerated.length > 0) {
          setDevices(enumerated);
          // If activeDeviceId is default, pick default device
          if (activeDeviceId === "default") {
            const defaultDev = enumerated.find((d) => d.is_default) || enumerated[0];
            setActiveDeviceId(defaultDev.id);
          }
        }
      } catch {
        // Fallback for browser preview
      }
    };
    fetchDevices();
  }, [activeDeviceId]);

  // 3. Listen to tray context menu actions from Tauri backend (FR28)
  useEffect(() => {
    let unlistenPower: (() => void) | undefined;
    let unlistenMute: (() => void) | undefined;
    let unlistenPreset: (() => void) | undefined;

    const setupListeners = async () => {
      try {
        unlistenPower = await listen<boolean>("tray-power-toggled", (event) => {
          setMasterPower(event.payload);
        });
        unlistenMute = await listen<boolean>("tray-mute-toggled", (event) => {
          setIsMuted(event.payload);
        });
        unlistenPreset = await listen<string>("tray-preset-selected", (event) => {
          setActiveEqPreset(event.payload);
          setNotification(`Preset activated: ${event.payload}`);
        });
      } catch {
        // Browser mode fallback
      }
    };

    setupListeners();

    return () => {
      if (unlistenPower) unlistenPower();
      if (unlistenMute) unlistenMute();
      if (unlistenPreset) unlistenPreset();
    };
  }, []);

  // 4. Auto-save configuration to atomic storage on user settings change (FR29, AD-10)
  useEffect(() => {
    if (!hasLoadedConfig) return;

    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
    }

    saveTimeoutRef.current = setTimeout(async () => {
      let windowX: number | null = null;
      let windowY: number | null = null;
      try {
        const win = getCurrentWindow();
        const pos = await win.outerPosition();
        windowX = pos.x;
        windowY = pos.y;
      } catch {
        // Browser mode fallback
      }

      const linearVol = volume <= 1.0 ? volume * volume : 1.0;
      const boost = volume > 1.0 ? volume : 1.0;

      const configPayload = {
        master_power: masterPower,
        master_volume: linearVol,
        master_boost: boost,
        is_muted: isMuted,
        active_output_device: activeDeviceId === "default" ? null : activeDeviceId,
        window_width: 840,
        window_height: 560,
        window_x: windowX,
        window_y: windowY,
        graphic_eq_gains_db: eqGains,
        surround_enabled: surroundEnabled,
        surround_intensity: surroundIntensity,
        speaker_muted: speakerMuted,
        speaker_gains_db: speakerGainsDb,
        speaker_azimuths_deg: speakerAzimuthsDeg,
        bypass_active: bypassActive,
        launch_on_startup: launchOnStartup,
      };

      try {
        await invoke("save_app_config", { config: configPayload });
      } catch (err) {
        console.error("Failed to persist app config:", err);
      }
    }, 250);

    return () => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
    };
  }, [
    hasLoadedConfig,
    masterPower,
    volume,
    isMuted,
    activeDeviceId,
    surroundEnabled,
    surroundIntensity,
    speakerMuted,
    speakerGainsDb,
    speakerAzimuthsDeg,
    bypassActive,
    launchOnStartup,
    eqGains,
  ]);

  const handleTogglePower = () => {
    const nextPower = !masterPower;
    setMasterPower(nextPower);
    dispatchCommand({ SetMasterPower: nextPower });
  };

  const handleVolumeChange = (newVol: number) => {
    const clampedSlider = Math.max(0, Math.min(1.5, newVol));
    setVolume(clampedSlider);
    if (isMuted && clampedSlider > 0) {
      setIsMuted(false);
    }
    const linearVol = clampedSlider <= 1.0 ? clampedSlider * clampedSlider : 1.0;
    const boost = clampedSlider > 1.0 ? clampedSlider : 1.0;
    dispatchCommand({ SetMasterVolume: { volume: linearVol, boost } });
  };

  const handleToggleMute = () => {
    const nextMute = !isMuted;
    setIsMuted(nextMute);
    dispatchCommand({ SetMute: nextMute });
  };

  const handleSelectDevice = async (deviceId: string) => {
    setActiveDeviceId(deviceId);
    try {
      await invoke("set_audio_device", { deviceId });
    } catch {
      setNotification(
        "Audio endpoint initialization failed. Falling back to Windows default endpoint."
      );
      try {
        await invoke("trigger_device_fallback");
        const defaultDev = devices.find((d) => d.is_default) || devices[0];
        if (defaultDev) {
          setActiveDeviceId(defaultDev.id);
        }
      } catch {
        // Fallback
      }
    }
  };

  const handleToggleSurround = () => {
    const nextSurround = !surroundEnabled;
    setSurroundEnabled(nextSurround);
    dispatchCommand({
      SetSurround3D: { enabled: nextSurround, intensity: surroundIntensity },
    });
  };

  const handleSurroundIntensityChange = (val: number) => {
    setSurroundIntensity(val);
    dispatchCommand({
      SetSurround3D: { enabled: surroundEnabled, intensity: val },
    });
  };

  const handleToggleEffect = (id: number) => {
    setEffectModules((prev) =>
      prev.map((mod) => {
        if (mod.id === id) {
          const nextEnabled = !mod.enabled;
          dispatchCommand({
            SetSweetener: {
              module_id: id,
              enabled: nextEnabled,
              amount: mod.amount,
            },
          });
          return { ...mod, enabled: nextEnabled };
        }
        return mod;
      })
    );
  };

  const handleEffectAmountChange = (id: number, val: number) => {
    setEffectModules((prev) =>
      prev.map((mod) => {
        if (mod.id === id) {
          dispatchCommand({
            SetSweetener: {
              module_id: id,
              enabled: mod.enabled,
              amount: val,
            },
          });
          return { ...mod, amount: val };
        }
        return mod;
      })
    );
  };

  const handleToggleBypass = () => {
    if (!masterPower) return;
    const nextBypass = !bypassActive;
    setBypassActive(nextBypass);
    dispatchCommand({ SetBypass: nextBypass });
  };

  const handleToggleStartup = async () => {
    const nextVal = !launchOnStartup;
    setLaunchOnStartup(nextVal);
    try {
      await invoke("set_launch_on_startup", { enabled: nextVal });
    } catch (err) {
      console.error("Failed to set startup status:", err);
      setNotification(`Failed to configure Windows startup: ${String(err)}`);
    }
  };

  // Virtual Soundstage Speaker Handlers (Story 2.3 & 2.4, FR7)
  const handleToggleSpeakerMute = (channelIndex: number) => {
    setSpeakerMuted((prev) => {
      const next = [...prev];
      next[channelIndex] = !next[channelIndex];
      dispatchCommand({
        SetSpeakerMute: { channel: channelIndex, muted: next[channelIndex] },
      });
      return next;
    });
  };

  const handleSpeakerGainChange = (channelIndex: number, gainDb: number) => {
    setSpeakerGainsDb((prev) => {
      const next = [...prev];
      next[channelIndex] = gainDb;
      dispatchCommand({
        SetSpeakerGain: { channel: channelIndex, gain_db: gainDb },
      });
      return next;
    });
  };

  const handleSpeakerAzimuthChange = (channelIndex: number, azimuthDeg: number) => {
    setSpeakerAzimuthsDeg((prev) => {
      const next = [...prev];
      next[channelIndex] = azimuthDeg;
      dispatchCommand({
        SetSpeakerAzimuth: { channel: channelIndex, azimuth_deg: azimuthDeg },
      });
      return next;
    });
  };

  const handleResetSpeakerPositions = () => {
    const defaultAzimuths = [-30, 30, 0, 180, -135, 135, -90, 90];
    setSpeakerAzimuthsDeg(defaultAzimuths);
    dispatchCommand("ResetSpeakerPositions");
  };

  const handleResetAllSpeakers = () => {
    const unmuted = [false, false, false, false, false, false, false, false];
    const zeroGains = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    setSpeakerMuted(unmuted);
    setSpeakerGainsDb(zeroGains);
    for (let i = 0; i < 8; i++) {
      dispatchCommand({ SetSpeakerMute: { channel: i, muted: false } });
      dispatchCommand({ SetSpeakerGain: { channel: i, gain_db: 0.0 } });
    }
  };

  // Global Keyboard Shortcuts (EXPERIENCE.md §6.3)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl + Shift + B: Toggle Master DSP Bypass (A/B Test)
      if (
        (e.ctrlKey || e.metaKey) &&
        e.shiftKey &&
        (e.key === "b" || e.key === "B" || e.code === "KeyB")
      ) {
        e.preventDefault();
        if (e.repeat) return;
        if (!masterPower) return;
        handleToggleBypass();
      }

      // Ctrl + Shift + S: Toggle Soundstage Configurator Modal
      if (
        (e.ctrlKey || e.metaKey) &&
        e.shiftKey &&
        (e.key === "s" || e.key === "S" || e.code === "KeyS")
      ) {
        e.preventDefault();
        if (e.repeat) return;
        setIsConfiguratorOpen((prev) => !prev);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [bypassActive, masterPower]);

  // Poll real-time telemetry from Tauri core
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const snapshot = await invoke<{
          limiter_gain_reduction_db: number;
          round_trip_latency_ms: number;
          dsp_active: boolean;
          limiter_headroom_db: number;
          sample_rate: number;
          bit_depth: number;
          limiter_active: boolean;
          using_virtual_bridge: boolean;
        }>("get_telemetry");
        if (snapshot) {
          setLatencyMs(snapshot.round_trip_latency_ms);
          setLimiterGainReductionDb(snapshot.limiter_gain_reduction_db);
          if (typeof snapshot.limiter_headroom_db === "number") {
            setLimiterHeadroomDb(snapshot.limiter_headroom_db);
          }
          if (snapshot.sample_rate) {
            setSampleRate(snapshot.sample_rate);
          }
          if (snapshot.bit_depth) {
            setBitDepth(snapshot.bit_depth);
          }
          if (typeof snapshot.limiter_active === "boolean") {
            setLimiterActive(snapshot.limiter_active);
          }
          if (typeof snapshot.using_virtual_bridge === "boolean") {
            setUsingVirtualBridge(snapshot.using_virtual_bridge);
          }
        }
      } catch {
        // Fallback for browser mode
      }
    }, 60);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="w-full h-full min-w-[780px] min-h-[520px] bg-background flex flex-col justify-between overflow-hidden border border-border-subtle shadow-2xl relative">
      {/* Settings Modal (Story 1.7, FR29) */}
      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        launchOnStartup={launchOnStartup}
        onToggleStartup={handleToggleStartup}
      />

      {/* 8-Channel Virtual Speaker Configurator Modal (Story 2.3 & 2.4, FR7) */}
      <SoundstageConfiguratorModal
        isOpen={isConfiguratorOpen}
        onClose={() => setIsConfiguratorOpen(false)}
        speakerMuted={speakerMuted}
        speakerGainsDb={speakerGainsDb}
        speakerAzimuthsDeg={speakerAzimuthsDeg}
        onToggleMute={handleToggleSpeakerMute}
        onGainChange={handleSpeakerGainChange}
        onAzimuthChange={handleSpeakerAzimuthChange}
        onResetPositions={handleResetSpeakerPositions}
        onResetAll={handleResetAllSpeakers}
      />

      {/* Fallback Notification Banner */}
      <NotificationBanner
        message={notification}
        onDismiss={() => setNotification(null)}
      />

      {/* 1. Custom Frameless Draggable Titlebar */}
      <Titlebar
        masterPower={masterPower}
        onTogglePower={handleTogglePower}
        volume={volume}
        onVolumeChange={handleVolumeChange}
        isMuted={isMuted}
        onToggleMute={handleToggleMute}
        devices={devices}
        activeDeviceId={activeDeviceId}
        onSelectDevice={handleSelectDevice}
        onOpenSettings={() => setIsSettingsOpen(true)}
      />

      {/* 2. Main Studio Content Area */}
      <main className="flex-1 p-3 grid grid-cols-12 gap-3 overflow-hidden">
        {/* Left soundstage panel: 3D Surround Dial */}
        <section className="col-span-5 bg-surface-base/60 border border-border-subtle rounded-xl flex items-center justify-center p-2 relative overflow-hidden">
          <SoundstageDial
            enabled={masterPower && surroundEnabled && !bypassActive}
            intensity={surroundIntensity}
            onIntensityChange={handleSurroundIntensityChange}
            onToggle={handleToggleSurround}
            onOpenConfigurator={() => setIsConfiguratorOpen(true)}
          />
        </section>

        {/* Right rack panel: Dual EQ & Effects Rack */}
        <section className="col-span-7 flex flex-col gap-2.5 overflow-hidden">
          <div className="flex-1">
            <EqualizerPreview
              activePreset={activeEqPreset}
              bands={eqGains}
              preampOffsetDb={preampOffsetDb}
            />
          </div>
          <div className="flex-1">
            <EffectsRackPreview
              modules={effectModules}
              onToggleModule={handleToggleEffect}
              onAmountChange={handleEffectAmountChange}
            />
          </div>
        </section>
      </main>

      {/* 3. Bottom Status & Telemetry Dock */}
      <StatusBar
        bypassActive={bypassActive}
        onToggleBypass={handleToggleBypass}
        latencyMs={latencyMs}
        limiterGainReductionDb={limiterGainReductionDb}
        limiterHeadroomDb={limiterHeadroomDb}
        limiterActive={limiterActive}
        sampleRate={sampleRate}
        bitDepth={bitDepth}
        dspActive={masterPower && !bypassActive}
        masterPower={masterPower}
        usingVirtualBridge={usingVirtualBridge}
      />
    </div>
  );
};

export default App;
