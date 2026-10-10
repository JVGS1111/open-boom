import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Titlebar } from "./components/Titlebar";
import { SoundstageDial } from "./components/SoundstageDial";
import { EqualizerPreview } from "./components/EqualizerPreview";
import { EffectsRackPreview, EffectModule } from "./components/EffectsRackPreview";
import { StatusBar } from "./components/StatusBar";

export const App: React.FC = () => {
  // Master Engine State
  const [masterPower, setMasterPower] = useState(true);
  const [volume, setVolume] = useState(0.8);
  const [isMuted, setIsMuted] = useState(false);
  const [activeDevice] = useState("Headphones (Realtek Audio)");

  // 3D Spatializer State
  const [surroundEnabled, setSurroundEnabled] = useState(true);
  const [surroundIntensity, setSurroundIntensity] = useState(0.72);

  // Equalizer State (31 Bands)
  const [activeEqPreset] = useState("Harman Acoustic");
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

  // Helper to dispatch IPC command safely to Tauri
  const dispatchCommand = async (commandObj: unknown) => {
    try {
      await invoke("dispatch_command", { cmd: commandObj });
    } catch {
      // Fallback for browser preview
    }
  };

  const handleTogglePower = () => {
    const nextPower = !masterPower;
    setMasterPower(nextPower);
    dispatchCommand({ SetMasterPower: nextPower });
  };

  const handleVolumeChange = (newVol: number) => {
    setVolume(newVol);
    if (isMuted && newVol > 0) {
      setIsMuted(false);
    }
    const linearVol = Math.min(1.0, newVol);
    const boost = Math.max(1.0, newVol);
    dispatchCommand({ SetMasterVolume: { volume: linearVol, boost } });
  };

  const handleToggleMute = () => {
    const nextMute = !isMuted;
    setIsMuted(nextMute);
    dispatchCommand({ SetMute: nextMute });
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
    const nextBypass = !bypassActive;
    setBypassActive(nextBypass);
    dispatchCommand({ SetBypass: nextBypass });
  };

  // Poll real-time telemetry from Tauri core
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const snapshot = await invoke<{
          limiter_gain_reduction_db: number;
          round_trip_latency_ms: number;
          dsp_active: boolean;
        }>("get_telemetry");
        if (snapshot) {
          setLatencyMs(snapshot.round_trip_latency_ms);
          setLimiterGainReductionDb(snapshot.limiter_gain_reduction_db);
        }
      } catch {
        // Fallback for browser mode
      }
    }, 500);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="w-full h-full min-w-[780px] min-h-[520px] bg-background flex flex-col justify-between overflow-hidden border border-border-subtle shadow-2xl">
      {/* 1. Custom Frameless Draggable Titlebar */}
      <Titlebar
        masterPower={masterPower}
        onTogglePower={handleTogglePower}
        volume={volume}
        onVolumeChange={handleVolumeChange}
        isMuted={isMuted}
        onToggleMute={handleToggleMute}
        activeDevice={activeDevice}
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
        dspActive={masterPower}
      />
    </div>
  );
};

export default App;
