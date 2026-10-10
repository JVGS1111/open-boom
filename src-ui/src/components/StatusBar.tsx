import React from "react";
import { ShieldAlert, Zap, Cpu } from "lucide-react";

interface StatusBarProps {
  bypassActive: boolean;
  onToggleBypass: () => void;
  latencyMs: number;
  limiterGainReductionDb: number;
  dspActive: boolean;
}

export const StatusBar: React.FC<StatusBarProps> = ({
  bypassActive,
  onToggleBypass,
  latencyMs,
  limiterGainReductionDb,
  dspActive,
}) => {
  const isLimiting = limiterGainReductionDb < -0.1;

  return (
    <footer className="h-9 bg-surface-base border-t border-border-subtle flex items-center justify-between px-4 select-none shrink-0 text-xs">
      {/* Engine Status & Latency Badge */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-1.5 text-text-secondary">
          <Cpu className="w-3.5 h-3.5 text-accent-cyan" />
          <span className="font-mono text-[11px]">48 kHz</span>
          <span className="text-border-default">|</span>
          <span className="font-mono text-[11px]">32-Bit Float</span>
          <span className="text-border-default">|</span>
          <span className="text-[11px] text-text-muted">Pro Audio MMCSS</span>
        </div>

        <div className="h-3 w-px bg-border-subtle" />

        {/* Latency badge */}
        <div className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-surface-card border border-border-subtle text-[10px] font-mono text-status-online">
          <Zap className="w-3 h-3 text-status-online" />
          <span>{latencyMs.toFixed(1)} ms</span>
        </div>
      </div>

      {/* Limiter Telemetry & A/B Bypass */}
      <div className="flex items-center gap-3">
        {/* Terminal Brickwall Limiter Status */}
        <div
          className={`flex items-center gap-1.5 px-2 py-0.5 rounded border text-[10px] font-mono transition-colors ${
            isLimiting
              ? "bg-meter-yellow/15 border-meter-yellow text-meter-yellow shadow-[0_0_8px_rgba(250,204,21,0.3)] animate-pulse"
              : "bg-surface-card border-border-subtle text-text-secondary"
          }`}
          title="Terminal 2ms Lookahead Limiter clamped at -0.2 dBFS"
        >
          <ShieldAlert className="w-3 h-3" />
          <span>LIMITER: -0.2 dBFS</span>
          {isLimiting && (
            <span className="font-bold text-meter-red">
              ({limiterGainReductionDb.toFixed(1)} dB)
            </span>
          )}
        </div>

        <div className="h-3 w-px bg-border-subtle" />

        {/* 1-Click A/B Instant Bypass Switch */}
        <button
          type="button"
          onClick={onToggleBypass}
          className={`px-3 py-1 rounded font-semibold text-[11px] tracking-wider transition-all duration-150 cursor-pointer ${
            bypassActive
              ? "bg-status-warning text-black shadow-[0_0_10px_rgba(245,158,11,0.4)]"
              : dspActive
              ? "bg-surface-card hover:bg-surface-elevated text-text-primary border border-border-subtle hover:border-border-default"
              : "bg-surface-base text-text-muted opacity-50 cursor-not-allowed"
          }`}
          title="Instant 1-Click Phase-Continuous A/B Bypass"
        >
          {bypassActive ? "BYPASSED (RAW)" : "A/B BYPASS"}
        </button>
      </div>
    </footer>
  );
};
