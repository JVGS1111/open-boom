import React from "react";
import { ShieldAlert, Zap, Cpu, Radio } from "lucide-react";
import { useI18n } from "../i18n";

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
  const { t } = useI18n();
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
          className="flex items-center gap-1.5 px-2 py-0.5 rounded border text-[10px] font-mono bg-surface-card border-border-subtle text-text-muted opacity-60 shrink-0 whitespace-nowrap"
          title="Terminal Limiter Disabled (Master Power OFF)"
        >
          <ShieldAlert className="w-3 h-3 text-text-muted opacity-50 shrink-0" />
          <span className="shrink-0">{t.statusBar.limiterOff}</span>
        </div>
      );
    }

    if (bypassActive) {
      return (
        <div
          role="status"
          aria-live="polite"
          className="flex items-center gap-1.5 px-2 py-0.5 rounded border text-[10px] font-mono bg-status-warning/10 border-status-warning/40 text-status-warning shrink-0 whitespace-nowrap"
          title="Terminal Limiter Inactive (A/B Bypass Engaged)"
        >
          <ShieldAlert className="w-3 h-3 text-status-warning shrink-0" />
          <span className="shrink-0">{t.statusBar.limiterBypassed}</span>
        </div>
      );
    }

    return (
      <div
        role="status"
        aria-live="polite"
        className={`flex items-center gap-1.5 px-2 py-0.5 rounded border text-[10px] font-mono transition-colors shrink-0 whitespace-nowrap ${
          isLimiting
            ? "bg-meter-yellow/20 border-meter-yellow text-meter-yellow shadow-[0_0_12px_rgba(250,204,21,0.4)] animate-pulse"
            : "bg-surface-card border-border-subtle text-text-secondary"
        }`}
        title={t.statusBar.limiterCeilingTooltip}
      >
        <ShieldAlert className={`w-3 h-3 shrink-0 ${isLimiting ? "text-meter-red" : "text-text-muted"}`} />
        <span className="shrink-0">{t.statusBar.limiterCeiling}</span>
        {isLimiting && (
          <span className="font-bold text-meter-red ml-0.5 shrink-0">
            ({limiterGainReductionDb.toFixed(1)} dB)
          </span>
        )}
      </div>
    );
  };

  return (
    <footer className="h-8 bg-surface-base border-t border-border-subtle flex items-center justify-between px-3 select-none shrink-0 text-xs overflow-hidden">
      {/* Engine Status & Latency Badge */}
      <div className="flex items-center gap-2 sm:gap-2.5 shrink-0 whitespace-nowrap min-w-0">
        {bypassActive ? (
          <div
            role="status"
            aria-live="polite"
            className="flex items-center gap-1.5 text-text-muted shrink-0"
            title="A/B Bypass Active (Hardware Passthrough with 2ms Limiter Protection)"
          >
            <Cpu className="w-3.5 h-3.5 text-text-muted shrink-0" />
            <span className="font-mono text-[11px] text-text-muted font-medium shrink-0">
              {t.statusBar.bypassBypassed}
            </span>
          </div>
        ) : (
          <div className="flex items-center gap-1.5 text-text-secondary shrink-0">
            <Cpu className="w-3.5 h-3.5 text-accent-cyan shrink-0" />
            <span className="font-mono text-[11px] text-text-primary font-medium shrink-0 whitespace-nowrap">
              {sampleRateText}
            </span>
            <span className="text-border-default shrink-0 hidden lg:inline">|</span>
            <span className="font-mono text-[11px] text-text-primary font-medium shrink-0 hidden lg:inline whitespace-nowrap">
              {bitDepthText}
            </span>
            <span className="text-border-default shrink-0 hidden xl:inline">|</span>
            <span className="text-[11px] text-text-muted shrink-0 hidden xl:inline whitespace-nowrap">{t.statusBar.proAudioMmmss}</span>
          </div>
        )}

        <div className="h-3 w-px bg-border-subtle shrink-0" />

        {/* Latency badge */}
        <div
          className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-surface-card border border-border-subtle text-[10px] font-mono text-status-online shrink-0 whitespace-nowrap"
          title={t.statusBar.latencyGuaranteed}
        >
          <Zap className="w-3 h-3 text-status-online shrink-0" />
          <span className="shrink-0 whitespace-nowrap tabular-nums">{latencyMs.toFixed(1)} ms</span>
        </div>

        <div className="h-3 w-px bg-border-subtle shrink-0" />

        {/* Virtual Bridge Capture Topology Badge (Story 1.8) */}
        {usingVirtualBridge ? (
          <div
            role="status"
            aria-live="polite"
            className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-status-online/10 border border-status-online/40 text-[10px] font-mono text-status-online shadow-[0_0_8px_rgba(34,197,94,0.15)] shrink-0 whitespace-nowrap select-none"
            title={t.statusBar.virtualSinkTooltip}
          >
            <Radio className="w-3 h-3 text-status-online shrink-0" />
            <span className="font-medium tracking-wide shrink-0 whitespace-nowrap">
              <span className="hidden sm:inline">{t.statusBar.virtualSink}</span>
              <span className="inline sm:hidden">{t.statusBar.virtualSink}</span>
            </span>
          </div>
        ) : (
          <div
            role="status"
            aria-live="polite"
            className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-status-warning/10 border border-status-warning/40 text-[10px] font-mono text-status-warning shrink-0 whitespace-nowrap select-none"
            title={t.statusBar.loopbackModeTooltip}
          >
            <Radio className="w-3 h-3 text-status-warning opacity-80 shrink-0" />
            <span className="font-medium tracking-wide shrink-0 whitespace-nowrap">{t.statusBar.loopbackMode}</span>
          </div>
        )}
      </div>

      {/* Limiter Telemetry & A/B Bypass */}
      <div className="flex items-center gap-2 sm:gap-2.5 shrink-0 whitespace-nowrap">
        {/* Dynamic Headroom Meter */}
        <div
          className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-surface-card border border-border-subtle text-[10px] font-mono shrink-0 whitespace-nowrap select-none"
          title={t.statusBar.headroomTooltip}
        >
          <span className="text-text-muted whitespace-nowrap shrink-0">{t.statusBar.headroom}</span>
          <span
            className={`inline-block min-w-[56px] text-right font-mono font-medium whitespace-nowrap tabular-nums shrink-0 ${
              isLimiting
                ? "text-meter-yellow font-bold"
                : "text-text-primary font-medium"
            }`}
          >
            {formatHeadroom(limiterHeadroomDb, isLimiting)}
          </span>
        </div>

        {/* Terminal Brickwall Limiter Status with ARIA role="status" and state handling */}
        {renderLimiterBadge()}

        <div className="h-3 w-px bg-border-subtle shrink-0" />

        {/* 1-Click A/B Instant Bypass Switch */}
        <button
          type="button"
          disabled={!masterPower}
          onClick={onToggleBypass}
          className={`px-2.5 sm:px-3 py-1 rounded font-semibold text-[11px] tracking-wider transition-all duration-150 shrink-0 whitespace-nowrap ${
            !masterPower
              ? "bg-surface-base text-text-muted opacity-50 cursor-not-allowed"
              : bypassActive
              ? "bg-status-warning text-black shadow-[0_0_10px_rgba(245,158,11,0.4)] cursor-pointer"
              : "bg-surface-card hover:bg-surface-elevated text-text-primary border border-border-subtle hover:border-border-default cursor-pointer"
          }`}
          title={t.statusBar.abBypassTooltip}
        >
          {bypassActive ? t.statusBar.abBypassedButton : t.statusBar.abBypassButton}
        </button>
      </div>
    </footer>
  );
};
