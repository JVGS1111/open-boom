import React, { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Titlebar } from "./components/Titlebar";
import { SoundstageDial } from "./components/SoundstageDial";
import { EqualizerPreview } from "./components/EqualizerPreview";
import { EffectsRackStrip, EffectModule } from "./components/EffectsRackStrip";
import { StatusBar } from "./components/StatusBar";
import { NotificationBanner } from "./components/NotificationBanner";
import { AudioDeviceInfo } from "./components/DeviceSelector";
import { SettingsModal } from "./components/SettingsModal";
import { SoundstageConfiguratorModal } from "./components/SoundstageConfiguratorModal";
import { EqualizerStudioDrawer } from "./components/EqualizerStudioDrawer";
import { AutoEqPopover, AutoEqProfileSummary } from "./components/AutoEqPopover";
import { AppMixerDrawer, ProcessAudioSession } from "./components/AppMixerDrawer";
import {
  PresetSelectorPopover,
  EqPreset,
  FACTORY_EQ_PRESETS,
  isPresetDirty,
  toPresetSlug,
} from "./components/PresetSelectorPopover";
import { I18nProvider, LanguageOption } from "./i18n";

/**
 * Calculates automatic negative preamp attenuation offset from positive filter gains:
 * A_preamp = -max(0, max_i(G_i)) in dB (FR15, AD-6).
 */
export function calculateAutoEqPreampOffset(
  filters: { gain_db?: number }[]
): number {
  let maxPositiveGain = 0;
  for (const f of filters) {
    const g = typeof f.gain_db === "number" && !isNaN(f.gain_db) ? f.gain_db : 0;
    if (g > maxPositiveGain) {
      maxPositiveGain = g;
    }
  }
  return maxPositiveGain > 0 ? -Math.round(maxPositiveGain * 10) / 10 : 0.0;
}

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

  // 8D Orbital Audio State (Story 6.2, CAP-4, CAP-5)
  const [orbitEnabled, setOrbitEnabled] = useState(false);
  const [orbitPhaseDeg, setOrbitPhaseDeg] = useState(0.0);

  // Equalizer State (31 Bands), Presets & Studio Drawer (Story 3.1, 3.2 & 3.5, FR9, FR11, FR12, UX-DR7)
  const [eqPresets, setEqPresets] = useState<EqPreset[]>(FACTORY_EQ_PRESETS);
  const [activeEqPreset, setActiveEqPreset] = useState("Flat");
  const [eqGains, setEqGains] = useState<number[]>(() => new Array(31).fill(0.0));
  const [preampOffsetDb, setPreampOffsetDb] = useState(0.0);
  const [isEqDrawerOpen, setIsEqDrawerOpen] = useState(false);
  const [isPresetSelectorOpen, setIsPresetSelectorOpen] = useState(false);
  const [eqMode, setEqMode] = useState<"10" | "31">("10");

  // AutoEq Headphone Profile Calibration State (Story 3.3 & 3.4, UX-DR8, FR15, AD-6)
  const [isAutoEqOpen, setIsAutoEqOpen] = useState(false);
  const [activeAutoEqProfile, setActiveAutoEqProfile] = useState<AutoEqProfileSummary | null>(null);

  // Application Stream Mixer State (Story 5.2, UX-DR9, FR21, FR22)
  const [isAppMixerOpen, setIsAppMixerOpen] = useState(false);
  const [audioSessions, setAudioSessions] = useState<ProcessAudioSession[]>([
    {
      process_id: 1001,
      name: "spotify.exe",
      volume: 0.85,
      is_muted: false,
      is_active: true,
    },
    {
      process_id: 1002,
      name: "discord.exe",
      volume: 0.70,
      is_muted: false,
      is_active: true,
    },
    {
      process_id: 1003,
      name: "chrome.exe",
      volume: 1.00,
      is_muted: false,
      is_active: false,
    },
  ]);

  // Language & i18n State
  const [language, setLanguage] = useState<LanguageOption>("system");

  // Sweeteners Rack State (Epic 4, UX-DR6)
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
      amount: 0.50,
    },
    {
      id: 3,
      name: "Spatial",
      description: "Mid/Side Stereo Crossfeed",
      enabled: true,
      amount: 0.60,
    },
    {
      id: 4,
      name: "Pitch",
      description: "Pitch Shifter (-12..+12 st)",
      enabled: false,
      amount: 0.00,
      isComingSoon: true,
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
  const [bufferGuardEnabled, setBufferGuardEnabled] = useState(true);
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

  // 1. Restore configuration, presets, and Windows startup settings on mount (FR11, FR29)
  useEffect(() => {
    const restoreConfig = async () => {
      try {
        const loadedPresets = await invoke<EqPreset[]>("get_eq_presets");
        if (loadedPresets && loadedPresets.length >= 15) {
          setEqPresets(loadedPresets);
        }
      } catch {
        // Browser mode fallback uses FACTORY_EQ_PRESETS
      }

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
          autoeq_profile_id?: string | null;
          active_preset_name?: string;
          sweetener_ambience_enabled?: boolean;
          sweetener_ambience_amount?: number;
          sweetener_fidelity_enabled?: boolean;
          sweetener_fidelity_amount?: number;
          sweetener_night_mode_enabled?: boolean;
          sweetener_night_mode_amount?: number;
          sweetener_spatial_enabled?: boolean;
          sweetener_spatial_amount?: number;
          buffer_guard_enabled?: boolean;
          language?: string;
        }>("load_app_config");

        if (config) {
          if (config.active_preset_name) {
            setActiveEqPreset(config.active_preset_name);
          }

          if (config.autoeq_profile_id) {
            try {
              const p = await invoke<{
                id: string;
                brand: string;
                model: string;
                name: string;
                target_curve: string;
                preamp_gain_db: number;
                filters: { filter_type: string; frequency_hz: number; q: number; gain_db: number }[];
              } | null>("get_autoeq_profile", { id: config.autoeq_profile_id });
              if (p) {
                setActiveAutoEqProfile({
                  id: p.id,
                  brand: p.brand,
                  model: p.model,
                  name: p.name,
                  target_curve: p.target_curve,
                  preamp_gain_db: p.preamp_gain_db,
                  filter_count: p.filters.length,
                  score: 100,
                });
                const restoredOffset = calculateAutoEqPreampOffset(p.filters);
                setPreampOffsetDb(restoredOffset);
                try {
                  await invoke("apply_autoeq_profile", {
                    profileId: p.id,
                    preampOffsetDb: restoredOffset,
                  });
                } catch {
                  // Graceful fallback
                }
              }
            } catch {
              // Graceful fallback
            }
          }

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

          if (config.graphic_eq_gains_db && config.graphic_eq_gains_db.length === 31) {
            setEqGains(config.graphic_eq_gains_db);
            dispatchCommand({
              LoadGraphicEqSnapshot: {
                gains_db: config.graphic_eq_gains_db,
                preamp_offset_db: 0.0,
              },
            });
          }

          setBypassActive(config.bypass_active);
          dispatchCommand({ SetBypass: config.bypass_active });

          setLaunchOnStartup(config.launch_on_startup);

          // Restore Sweeteners Rack (Epic 4, UX-DR6, FR16-FR19)
          const ambEn = config.sweetener_ambience_enabled ?? true;
          const ambAmt = config.sweetener_ambience_amount ?? 0.35;
          const fidEn = config.sweetener_fidelity_enabled ?? true;
          const fidAmt = config.sweetener_fidelity_amount ?? 0.45;
          const nmEn = config.sweetener_night_mode_enabled ?? false;
          const nmAmt = config.sweetener_night_mode_amount ?? 0.50;
          const spatEn = config.sweetener_spatial_enabled ?? true;
          const spatAmt = config.sweetener_spatial_amount ?? 0.60;

          setEffectModules([
            { id: 0, name: "Ambience", description: "FDN Reverb Matrix", enabled: ambEn, amount: ambAmt },
            { id: 1, name: "Fidelity", description: "Harmonic Clarity Exciter", enabled: fidEn, amount: fidAmt },
            { id: 2, name: "Night Mode", description: "Dynamic Range Soft-Knee", enabled: nmEn, amount: nmAmt },
            { id: 3, name: "Spatial", description: "Mid/Side Stereo Crossfeed", enabled: spatEn, amount: spatAmt },
            { id: 4, name: "Pitch", description: "Pitch Shifter (-12..+12 st)", enabled: false, amount: 0.00, isComingSoon: true },
          ]);

          dispatchCommand({ SetSweetener: { module_id: 0, enabled: ambEn, amount: ambAmt } });
          dispatchCommand({ SetSweetener: { module_id: 1, enabled: fidEn, amount: fidAmt } });
          dispatchCommand({ SetSweetener: { module_id: 2, enabled: nmEn, amount: nmAmt } });
          dispatchCommand({ SetSweetener: { module_id: 3, enabled: spatEn, amount: spatAmt } });

          // Restore Audio Buffer Stability Guard
          const bgEn = config.buffer_guard_enabled ?? true;
          setBufferGuardEnabled(bgEn);
          dispatchCommand({ SetBufferGuard: bgEn });

          // Restore Language Preference
          if (config.language) {
            setLanguage(config.language as LanguageOption);
          }
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
    let unlistenLanguage: (() => void) | undefined;

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
          const matched = FACTORY_EQ_PRESETS.find(
            (p) => p.name.toLowerCase() === event.payload.toLowerCase()
          );
          if (matched) {
            setEqGains([...matched.gains_db]);
          }
          setNotification(`Preset activated: ${event.payload}`);
        });
        unlistenLanguage = await listen<string>("tray-language-changed", (event) => {
          setLanguage(event.payload as LanguageOption);
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
      if (unlistenLanguage) unlistenLanguage();
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
        window_width: 1040,
        window_height: 640,
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
        autoeq_profile_id: activeAutoEqProfile?.id ?? null,
        active_preset_name: activeEqPreset,
        sweetener_ambience_enabled: effectModules.find((m) => m.id === 0)?.enabled ?? true,
        sweetener_ambience_amount: effectModules.find((m) => m.id === 0)?.amount ?? 0.35,
        sweetener_fidelity_enabled: effectModules.find((m) => m.id === 1)?.enabled ?? true,
        sweetener_fidelity_amount: effectModules.find((m) => m.id === 1)?.amount ?? 0.45,
        sweetener_night_mode_enabled: effectModules.find((m) => m.id === 2)?.enabled ?? false,
        sweetener_night_mode_amount: effectModules.find((m) => m.id === 2)?.amount ?? 0.50,
        sweetener_spatial_enabled: effectModules.find((m) => m.id === 3)?.enabled ?? true,
        sweetener_spatial_amount: effectModules.find((m) => m.id === 3)?.amount ?? 0.60,
        sweetener_pitch_semitones: 0,
        buffer_guard_enabled: bufferGuardEnabled,
        language: language,
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
    bufferGuardEnabled,
    eqGains,
    activeAutoEqProfile,
    activeEqPreset,
    effectModules,
    language,
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

  const handleToggleOrbit = async () => {
    const nextOrbit = !orbitEnabled;
    setOrbitEnabled(nextOrbit);

    // Auto-engage 3D spatializer if disabled (CAP-4, Story 6.2 Edge Case Matrix)
    if (nextOrbit && !surroundEnabled) {
      setSurroundEnabled(true);
      dispatchCommand({
        SetSurround3D: { enabled: true, intensity: surroundIntensity },
      });
    }

    try {
      await invoke("set_orbit_8d", { enabled: nextOrbit });
    } catch {
      // Browser preview fallback
      dispatchCommand({
        SetOrbit8D: {
          enabled: nextOrbit,
          direction: "Clockwise",
          period_sec: 12.0,
          depth: 0.85,
        },
      });
    }
  };

  const handleToggleEffect = (id: number) => {
    if (id >= 4) return; // Pitch is Phase 2 Coming Soon (FR20)
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
    if (id >= 4) return; // Pitch is Phase 2 Coming Soon (FR20)
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

  const handleToggleBufferGuard = async () => {
    const nextVal = !bufferGuardEnabled;
    setBufferGuardEnabled(nextVal);
    dispatchCommand({ SetBufferGuard: nextVal });
    try {
      await invoke("set_buffer_guard", { enabled: nextVal });
    } catch (err) {
      console.error("Failed to set buffer guard status:", err);
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

  // Equalizer Studio & Preset Handlers (Story 3.2 & 3.5, FR9, FR11, FR12, UX-DR7)
  const activePresetObj =
    eqPresets.find((p) => p.name.toLowerCase() === activeEqPreset.toLowerCase()) ||
    FACTORY_EQ_PRESETS.find((p) => p.name.toLowerCase() === activeEqPreset.toLowerCase()) ||
    FACTORY_EQ_PRESETS[0];
  const presetDirty = isPresetDirty(eqGains, activePresetObj.gains_db);

  const handleEqBandGainChange = (bandIndex: number, gainDb: number) => {
    const clampedGain = Math.max(-12.0, Math.min(12.0, Math.round(gainDb * 10) / 10));
    setEqGains((prev) => {
      const next = [...prev];
      next[bandIndex] = clampedGain;
      return next;
    });
    dispatchCommand({
      SetEqBandGain: { band_idx: bandIndex, gain_db: clampedGain },
    });
  };

  const handleResetEqGains = () => {
    const zeroGains = new Array(31).fill(0.0);
    setActiveEqPreset("Flat");
    setEqGains(zeroGains);
    dispatchCommand({
      LoadGraphicEqSnapshot: {
        gains_db: zeroGains,
        preamp_offset_db: 0.0,
      },
    });
  };

  const handleSelectEqPreset = (preset: EqPreset) => {
    const nextGains = preset.gains_db.slice(0, 31);
    while (nextGains.length < 31) {
      nextGains.push(0.0);
    }
    setActiveEqPreset(preset.name);
    setEqGains(nextGains);
    dispatchCommand({
      LoadGraphicEqSnapshot: {
        gains_db: nextGains,
        preamp_offset_db: 0.0,
      },
    });
  };

  const isTauriRuntime = () =>
    typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

  const sortPresetsList = (list: EqPreset[]): EqPreset[] => {
    const factory = list.filter((p) => p.is_factory);
    const custom = list
      .filter((p) => !p.is_factory)
      .sort((a, b) =>
        a.name.toLowerCase().localeCompare(b.name.toLowerCase()) ||
        a.id.localeCompare(b.id)
      );
    return [...factory, ...custom];
  };

  const handleSaveCustomPreset = async (name: string) => {
    const trimmed = name.trim();
    if (!trimmed) return;

    let savedPreset: EqPreset;
    try {
      savedPreset = await invoke<EqPreset>("save_custom_eq_preset", {
        name: trimmed,
        gainsDb: eqGains,
      });
      const updatedList = await invoke<EqPreset[]>("get_eq_presets");
      if (updatedList && updatedList.length >= 15) {
        setEqPresets(updatedList);
      }
    } catch (err) {
      if (isTauriRuntime()) {
        throw err;
      }
      // Browser preview fallback
      const slug = toPresetSlug(trimmed);
      savedPreset = {
        id: slug,
        name: trimmed,
        category: "Custom",
        is_factory: false,
        gains_db: [...eqGains],
      };
      setEqPresets((prev) => {
        const filtered = prev.filter((p) => p.id !== slug);
        return sortPresetsList([...filtered, savedPreset]);
      });
    }

    setActiveEqPreset(savedPreset.name);
    setNotification(`Custom preset saved: ${savedPreset.name}`);
  };

  const handleRenameCustomPreset = async (oldId: string, newName: string) => {
    const trimmed = newName.trim();
    if (!trimmed) return;

    const oldPreset = eqPresets.find((p) => p.id === oldId);
    let renamedPreset: EqPreset;

    try {
      renamedPreset = await invoke<EqPreset>("rename_custom_eq_preset", {
        oldId,
        newName: trimmed,
      });
      const updatedList = await invoke<EqPreset[]>("get_eq_presets");
      if (updatedList && updatedList.length >= 15) {
        setEqPresets(updatedList);
      }
    } catch (err) {
      if (isTauriRuntime()) {
        throw err;
      }
      // Browser preview fallback
      const slug = toPresetSlug(trimmed);
      renamedPreset = {
        id: slug,
        name: trimmed,
        category: "Custom",
        is_factory: false,
        gains_db: oldPreset ? [...oldPreset.gains_db] : [...eqGains],
      };
      setEqPresets((prev) =>
        sortPresetsList(prev.map((p) => (p.id === oldId ? renamedPreset : p)))
      );
    }

    if (
      oldPreset &&
      oldPreset.name.toLowerCase() === activeEqPreset.toLowerCase()
    ) {
      setActiveEqPreset(renamedPreset.name);
    }
    setNotification(`Preset renamed to: ${renamedPreset.name}`);
  };

  const handleDeleteCustomPreset = async (preset: EqPreset) => {
    if (preset.is_factory) return;

    try {
      await invoke("delete_custom_eq_preset", { id: preset.id });
      const updatedList = await invoke<EqPreset[]>("get_eq_presets");
      if (updatedList && updatedList.length >= 15) {
        setEqPresets(updatedList);
      }
    } catch (err) {
      if (isTauriRuntime()) {
        throw err;
      }
      // Browser preview fallback
      setEqPresets((prev) => prev.filter((p) => p.id !== preset.id));
    }

    if (preset.name.toLowerCase() === activeEqPreset.toLowerCase()) {
      setActiveEqPreset("Flat");
    }
    setNotification(`Deleted custom preset: ${preset.name}`);
  };

  // AutoEq Profile Selection Handlers (Story 3.3 & 3.4, UX-DR8, FR15, AD-6)
  const handleSelectAutoEqProfile = async (profile: AutoEqProfileSummary) => {
    setActiveAutoEqProfile(profile);
    setIsAutoEqOpen(false);

    let offset = 0.0;
    try {
      const fullProfile = await invoke<{
        id: string;
        brand: string;
        model: string;
        name: string;
        target_curve: string;
        preamp_gain_db: number;
        filters: { filter_type: string; frequency_hz: number; q: number; gain_db: number }[];
      } | null>("get_autoeq_profile", { id: profile.id });

      if (fullProfile) {
        offset = calculateAutoEqPreampOffset(fullProfile.filters);
        await invoke("apply_autoeq_profile", {
          profileId: profile.id,
          preampOffsetDb: offset,
        });
      }
    } catch {
      // In browser preview fallback, compute from summary if available
      offset = typeof profile.preamp_gain_db === "number" && profile.preamp_gain_db < 0
        ? profile.preamp_gain_db
        : 0.0;
    }

    setPreampOffsetDb(offset);
    setNotification(`AutoEq profile applied: ${profile.name} (${offset.toFixed(1)} dB preamp)`);
  };

  const handleClearAutoEqProfile = async () => {
    setActiveAutoEqProfile(null);
    setIsAutoEqOpen(false);
    setPreampOffsetDb(0.0);

    try {
      await invoke("clear_autoeq_profile");
    } catch {
      // Browser preview fallback
    }

    setNotification("AutoEq profile cleared. EQ restored to flat.");
  };

  // Per-Application Stream Mixer Handlers (Story 5.2, UX-DR9, FR21, FR22)
  const fetchAudioSessions = async () => {
    try {
      const list = await invoke<ProcessAudioSession[]>("get_audio_sessions");
      if (Array.isArray(list)) {
        setAudioSessions(list);
      }
    } catch {
      // Browser preview fallback maintains mock sessions
    }
  };

  const handleAppVolumeChange = async (processId: number, newVol: number) => {
    const clamped = Math.max(0, Math.min(1.5, newVol));
    setAudioSessions((prev) =>
      prev.map((s) => (s.process_id === processId ? { ...s, volume: clamped } : s))
    );
    try {
      await invoke("set_app_volume", { processId, volume: clamped });
    } catch {
      // Keep optimistic update in UI
    }
  };

  const handleAppToggleMute = async (processId: number, currentMuted: boolean) => {
    const nextMuted = !currentMuted;
    setAudioSessions((prev) =>
      prev.map((s) => (s.process_id === processId ? { ...s, is_muted: nextMuted } : s))
    );
    try {
      await invoke("set_app_mute", { processId, mute: nextMuted });
    } catch {
      // Keep optimistic update in UI
    }
  };

  const handleResetAppVolume = async (processName: string, processId: number) => {
    try {
      await invoke("reset_app_volume_profile", { processName });
    } catch {
      // Ignored in browser preview fallback
    }
    await handleAppVolumeChange(processId, 1.0);
    setAudioSessions((prev) =>
      prev.map((s) => (s.process_id === processId ? { ...s, is_muted: false } : s))
    );
    try {
      await invoke("set_app_mute", { processId, mute: false });
    } catch {
      // Ignored in browser preview fallback
    }
    setNotification(`Reset ${processName} volume to default 100% (0.0 dB)`);
  };

  // Poll active audio sessions whenever mixer drawer is open
  useEffect(() => {
    if (!isAppMixerOpen) return;

    fetchAudioSessions();
    const interval = setInterval(() => {
      fetchAudioSessions();
    }, 1000);

    return () => clearInterval(interval);
  }, [isAppMixerOpen]);

  // Global Keyboard Shortcuts (EXPERIENCE.md §6.3)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore global hotkeys if an editable input is focused
      const activeTag = document.activeElement?.tagName.toLowerCase();
      const isInputFocused =
        activeTag === "input" ||
        activeTag === "textarea" ||
        (document.activeElement as HTMLElement)?.isContentEditable;

      // Ctrl + Shift + B: Toggle Master DSP Bypass (A/B Test)
      if (
        (e.ctrlKey || e.metaKey) &&
        e.shiftKey &&
        (e.key === "b" || e.key === "B" || e.code === "KeyB")
      ) {
        if (isInputFocused) return;
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
        if (isInputFocused) return;
        e.preventDefault();
        if (e.repeat) return;
        setIsConfiguratorOpen((prev) => !prev);
      }

      // Ctrl + Shift + E: Toggle Equalizer Studio Drawer (Story 3.2, UX-DR7)
      if (
        (e.ctrlKey || e.metaKey) &&
        e.shiftKey &&
        (e.key === "e" || e.key === "E" || e.code === "KeyE")
      ) {
        if (isInputFocused) return;
        e.preventDefault();
        if (e.repeat) return;
        setIsEqDrawerOpen((prev) => !prev);
      }

      // Ctrl + Shift + M: Toggle Application Stream Mixer Drawer (Story 5.2, UX-DR9)
      if (
        (e.ctrlKey || e.metaKey) &&
        e.shiftKey &&
        (e.key === "m" || e.key === "M" || e.code === "KeyM")
      ) {
        if (isInputFocused) return;
        e.preventDefault();
        if (e.repeat) return;
        setIsAppMixerOpen((prev) => !prev);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [bypassActive, masterPower]);

  // Poll real-time telemetry from Tauri core at >= 30 fps (Story 6.2, CAP-5)
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
          orbit_phase_deg?: number;
          orbit_enabled?: boolean;
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
          if (typeof snapshot.orbit_phase_deg === "number") {
            setOrbitPhaseDeg(snapshot.orbit_phase_deg);
          }
          if (typeof snapshot.orbit_enabled === "boolean") {
            setOrbitEnabled(snapshot.orbit_enabled);
          }
        }
      } catch {
        // Fallback for browser mode
      }
    }, 33);

    return () => clearInterval(interval);
  }, []);

  return (
    <I18nProvider initialLanguage={language} onLanguageChange={setLanguage}>
      <div className="w-full h-full min-w-[960px] min-h-[580px] bg-background flex flex-col overflow-hidden border border-border-subtle relative select-none">
      {/* Settings Modal (Story 1.7, FR29) */}
      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        launchOnStartup={launchOnStartup}
        onToggleStartup={handleToggleStartup}
        bufferGuardEnabled={bufferGuardEnabled}
        onToggleBufferGuard={handleToggleBufferGuard}
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

      {/* AutoEq Headphone Profile Search Popover (Story 3.3, UX-DR8) */}
      <AutoEqPopover
        isOpen={isAutoEqOpen}
        onClose={() => setIsAutoEqOpen(false)}
        activeProfileId={activeAutoEqProfile?.id}
        onSelectProfile={handleSelectAutoEqProfile}
        onClearProfile={handleClearAutoEqProfile}
      />

      {/* Equalizer Genre Factory & Custom Presets Popover (Story 3.5, FR11, AD-10) */}
      <PresetSelectorPopover
        isOpen={isPresetSelectorOpen}
        onClose={() => setIsPresetSelectorOpen(false)}
        presets={eqPresets}
        activePresetName={activeEqPreset}
        currentGains={eqGains}
        isDirty={presetDirty}
        onSelectPreset={handleSelectEqPreset}
        onSaveCustomPreset={handleSaveCustomPreset}
        onRenameCustomPreset={handleRenameCustomPreset}
        onDeleteCustomPreset={handleDeleteCustomPreset}
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
        onToggleAppMixer={() => setIsAppMixerOpen((prev) => !prev)}
        isAppMixerOpen={isAppMixerOpen}
      />

      {/* 2. Main Studio Content Area */}
      <main className="flex-1 p-3 flex gap-3 overflow-hidden relative min-h-0">
        {/* Left soundstage panel: 3D Surround Dial */}
        <section className="w-[280px] shrink-0 bg-surface-base/60 border border-border-subtle rounded-xl flex items-center justify-center p-2 relative overflow-hidden">
          <SoundstageDial
            enabled={masterPower && surroundEnabled && !bypassActive}
            intensity={surroundIntensity}
            onIntensityChange={handleSurroundIntensityChange}
            onToggle={handleToggleSurround}
            onOpenConfigurator={() => setIsConfiguratorOpen(true)}
            orbitEnabled={masterPower && !bypassActive && surroundEnabled && orbitEnabled}
            orbitPhaseDeg={orbitPhaseDeg}
            onToggleOrbit={handleToggleOrbit}
          />
        </section>

        {/* Right rack panel: Dual EQ & Effects Rack (60% EQ / 40% Effects) */}
        <section className="flex-1 min-w-0 flex flex-col gap-2.5 overflow-hidden h-full">
          <div className="flex-[6] min-h-0 flex flex-col">
            <EqualizerPreview
              activePreset={activeEqPreset}
              isPresetDirty={presetDirty}
              bands={eqGains}
              preampOffsetDb={preampOffsetDb}
              onExpand={() => setIsEqDrawerOpen(true)}
              onOpenPresets={() => setIsPresetSelectorOpen(true)}
              activeAutoEqName={activeAutoEqProfile ? activeAutoEqProfile.name : undefined}
              onOpenAutoEq={() => setIsAutoEqOpen(true)}
            />
          </div>
          <div className="flex-[4] min-h-0 flex flex-col">
            <EffectsRackStrip
              modules={effectModules}
              onToggleModule={handleToggleEffect}
              onAmountChange={handleEffectAmountChange}
              disabled={!masterPower || bypassActive}
            />
          </div>
        </section>

        {/* Equalizer Studio Drawer (Story 3.2, UX-DR7) */}
        <EqualizerStudioDrawer
          isOpen={isEqDrawerOpen}
          onClose={() => setIsEqDrawerOpen(false)}
          mode={eqMode}
          onModeChange={setEqMode}
          gains={eqGains}
          onGainChange={handleEqBandGainChange}
          onResetAll={handleResetEqGains}
          activePreset={activeEqPreset}
          isPresetDirty={presetDirty}
          onOpenPresets={() => setIsPresetSelectorOpen(true)}
          preampOffsetDb={preampOffsetDb}
          activeAutoEqName={activeAutoEqProfile ? activeAutoEqProfile.name : undefined}
          onOpenAutoEq={() => setIsAutoEqOpen(true)}
        />

        {/* Application Stream Mixer Drawer (Story 5.2, UX-DR9) */}
        <AppMixerDrawer
          isOpen={isAppMixerOpen}
          onClose={() => setIsAppMixerOpen(false)}
          sessions={audioSessions}
          onVolumeChange={handleAppVolumeChange}
          onToggleMute={handleAppToggleMute}
          onResetProfile={handleResetAppVolume}
          onRefresh={fetchAudioSessions}
        />
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
  </I18nProvider>
  );
};

export default App;
