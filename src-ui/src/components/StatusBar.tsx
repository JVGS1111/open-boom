import React from "react";
import { ShieldAlert, Zap, Cpu, Radio } from "lucide-react";

interface StatusBarProps {
  bypassActive: boolean;
  onToggleBypass: () => void;
  latencyMs: number;
  limiterGainReductionDb: number;
  limiterHeadroomDb?: number;
  limiterActive?: boolean;
  sampleRate?: number;
  bitDepth?: number;
  dspActive: boolean;
  masterPower?: boolean;
  usingVirtualBridge?: boolean;
}

export const StatusBar: React.FC<StatusBarProps> = ({
  bypassActive,
  onToggleBypass,
  latencyMs,
  limiterGainReductionDb,
  limiterHeadroomDb = 20.0,
  limiterActive = false,
  sampleRate = 48000,
  bitDepth = 32,
  dspActive,
  masterPower = true,
  usingVirtualBridge = false,
}) => {
  const isEngaged = masterPower && !bypassActive && dspActive;
  const isLimiting = isEngaged && (limiterActive || limiterGainReductionDb < -0.05);

  const sampleRateText =
    sampleRate >= 1000
      ? `${(sampleRate / 1000).toFixed(sampleRate % 1000 === 0 ? 0 : 1)} kHz`
      : `${sampleRate} Hz`;

  const bitDepthText = `${bitDepth}-Bit Float`;

  const formatHeadroom = (headroom: number, limiting: boolean) => {
    if (limiting || headroom <= 0.05) {
      return "0.0 dB";
    }
    if (headroom >= 19.95) {
      return ">20 dB";
    }
    return `+${headroom.toFixed(1)} dB`;
  };

  const renderLimiterBadge = () => {
    if (!masterPower) {
      return (
        <div
          role="status"
          aria-live="polite"
          className="flex items-center gap-1.5 px-2 py-0.5 rounded border text-[10px] font-mono bg-surface-card border-border-subtle text-text-muted opacity-60"
          title="Terminal Limiter Disabled (Master Power OFF)"
        >
          <ShieldAlert className="w-3 h-3 text-text-muted opacity-50" />
          <span>LIMITER: OFF</span>
        </div>
      );
    }

    if (bypassActive) {
      return (
        <div
          role="status"
          aria-live="polite"
          className="flex items-center gap-1.5 px-2 py-0.5 rounded border text-[10px] font-mono bg-status-warning/10 border-status-warning/40 text-status-warning"
          title="Terminal Limiter Inactive (A/B Bypass Engaged)"
        >
          <ShieldAlert className="w-3 h-3 text-status-warning" />
          <span>LIMITER: BYPASSED</span>
        </div>
      );
    }

    return (
      <div
        role="status"
        aria-live="polite"
        className={`flex items-center gap-1.5 px-2 py-0.5 rounded border text-[10px] font-mono transition-colors ${
          isLimiting
            ? "bg-meter-yellow/20 border-meter-yellow text-meter-yellow shadow-[0_0_12px_rgba(250,204,21,0.4)] animate-pulse"
            : "bg-surface-card border-border-subtle text-text-secondary"
        }`}
        title="Terminal 2ms Lookahead Limiter clamped at -0.2 dBFS"
      >
        <ShieldAlert className={`w-3 h-3 ${isLimiting ? "text-meter-red" : "text-text-muted"}`} />
        <span>LIMITER: -0.2 dBFS</span>
        {isLimiting && (
          <span className="font-bold text-meter-red ml-0.5">
            ({limiterGainReductionDb.toFixed(1)} dB)
          </span>
        )}
      </div>
    );
  };

  return (
    <footer className="h-8 bg-surface-base border-t border-border-subtle flex items-center justify-between px-3.5 select-none shrink-0 text-xs">
      {/* Engine Status & Latency Badge */}
      <div className="flex items-center gap-3">
        {bypassActive ? (
          <div
            role="status"
            aria-live="polite"
            className="flex items-center gap-1.5 text-text-muted"
            title="A/B Bypass Active (Hardware Passthrough with 2ms Limiter Protection)"
          >
            <Cpu className="w-3.5 h-3.5 text-text-muted" />
            <span className="font-mono text-[11px] text-text-muted font-medium">
              Hardware Passthrough (Bypassed)
            </span>
          </div>
        ) : (
          <div className="flex items-center gap-1.5 text-text-secondary">
            <Cpu className="w-3.5 h-3.5 text-accent-cyan" />
            <span className="font-mono text-[11px] text-text-primary font-medium">
              {sampleRateText}
            </span>
            <span className="text-border-default">|</span>
            <span className="font-mono text-[11px] text-text-primary font-medium">
              {bitDepthText}
            </span>
            <span className="text-border-default">|</span>
            <span className="text-[11px] text-text-muted">Pro Audio MMCSS</span>
          </div>
        )}

        <div className="h-3 w-px bg-border-subtle" />

        {/* Latency badge */}
        <div
          className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-surface-card border border-border-subtle text-[10px] font-mono text-status-online"
          title="Round-trip latency < 15ms guaranteed"
        >
          <Zap className="w-3 h-3 text-status-online" />
          <span>{latencyMs.toFixed(1)} ms</span>
        </div>

        <div className="h-3 w-px bg-border-subtle" />

        {/* Virtual Bridge Capture Topology Badge (Story 1.8) */}
        {usingVirtualBridge ? (
          <div
            role="status"
            aria-live="polite"
            className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-status-online/10 border border-status-online/40 text-[10px] font-mono text-status-online shadow-[0_0_8px_rgba(34,197,94,0.15)]"
            title="Audio captured via Virtual Audio Device sink with zero slapback echo (Mode A)"
          >
            <Radio className="w-3 h-3 text-status-online" />
            <span className="font-medium tracking-wide">Virtual Sink (Echo-Free)</span>
          </div>
        ) : (
          <div
            role="status"
            aria-live="polite"
            className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-status-warning/10 border border-status-warning/40 text-[10px] font-mono text-status-warning"
            title="Audio captured via driverless WASAPI loopback with process exclusion (Mode B)"
          >
            <Radio className="w-3 h-3 text-status-warning opacity-80" />
            <span className="font-medium tracking-wide">Loopback Mode</span>
          </div>
        )}
      </div>

      {/* Limiter Telemetry & A/B Bypass */}
      <div className="flex items-center gap-3">
        {/* Dynamic Headroom Meter */}
        <div
          className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-surface-card border border-border-subtle text-[10px] font-mono"
          title="Dynamic Headroom to -0.2 dBFS Terminal Limiter Ceiling"
        >
          <span className="text-text-muted">Headroom:</span>
          <span
            className={
              isLimiting
                ? "text-meter-yellow font-bold"
                : "text-text-primary font-medium"
            }
          >
            {formatHeadroom(limiterHeadroomDb, isLimiting)}
          </span>
        </div>

        {/* Terminal Brickwall Limiter Status with ARIA role="status" and state handling */}
        {renderLimiterBadge()}

        <div className="h-3 w-px bg-border-subtle" />

        {/* 1-Click A/B Instant Bypass Switch */}
        <button
          type="button"
          disabled={!masterPower}
          onClick={onToggleBypass}
          className={`px-3 py-1 rounded font-semibold text-[11px] tracking-wider transition-all duration-150 ${
            !masterPower
              ? "bg-surface-base text-text-muted opacity-50 cursor-not-allowed"
              : bypassActive
              ? "bg-status-warning text-black shadow-[0_0_10px_rgba(245,158,11,0.4)] cursor-pointer"
              : "bg-surface-card hover:bg-surface-elevated text-text-primary border border-border-subtle hover:border-border-default cursor-pointer"
          }`}
          title="Instant 1-Click Phase-Continuous A/B Bypass (Ctrl + Shift + B)"
        >
          {bypassActive ? "BYPASSED (RAW)" : "A/B BYPASS"}
        </button>
      </div>
    </footer>
  );
};
