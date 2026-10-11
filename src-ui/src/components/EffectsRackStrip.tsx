import React from "react";
import { Sparkles, Waves, Moon, Expand, Music2 } from "lucide-react";
import { useI18n } from "../i18n";

export interface EffectModule {
  id: number;
  name: string;
  description: string;
  enabled: boolean;
  amount: number;
  isComingSoon?: boolean;
}

interface EffectsRackStripProps {
  modules: EffectModule[];
  onToggleModule: (id: number) => void;
  onAmountChange: (id: number, val: number) => void;
  disabled?: boolean;
}

const DEFAULT_AMOUNTS: Record<number, number> = {
  0: 0.35, // Ambience
  1: 0.45, // Fidelity
  2: 0.50, // Night Mode
  3: 0.60, // Spatial
  4: 0.00, // Pitch (0 st)
};

export const EffectsRackStrip: React.FC<EffectsRackStripProps> = ({
  modules,
  onToggleModule,
  onAmountChange,
  disabled = false,
}) => {
  const { t } = useI18n();

  const getModuleName = (id: number, fallback: string) => {
    switch (id) {
      case 0: return t.effectsRack.ambienceTitle;
      case 1: return t.effectsRack.fidelityTitle;
      case 2: return t.effectsRack.nightModeTitle;
      case 3: return t.effectsRack.spatialTitle;
      case 4: return t.effectsRack.pitchTitle;
      default: return fallback;
    }
  };

  const getModuleDescription = (id: number, fallback: string) => {
    switch (id) {
      case 0: return t.effectsRack.ambienceDesc;
      case 1: return t.effectsRack.fidelityDesc;
      case 2: return t.effectsRack.nightModeDesc;
      case 3: return t.effectsRack.spatialDesc;
      case 4: return t.effectsRack.pitchDesc;
      default: return fallback;
    }
  };

  const getIcon = (id: number) => {
    switch (id) {
      case 0:
        return <Waves className="w-3.5 h-3.5" />;
      case 1:
        return <Sparkles className="w-3.5 h-3.5" />;
      case 2:
        return <Moon className="w-3.5 h-3.5" />;
      case 3:
        return <Expand className="w-3.5 h-3.5" />;
      case 4:
      default:
        return <Music2 className="w-3.5 h-3.5" />;
    }
  };

  const handleKeyDown = (
    e: React.KeyboardEvent,
    mod: EffectModule
  ) => {
    if (disabled || mod.isComingSoon) return;

    if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      onToggleModule(mod.id);
    } else if (e.key === "ArrowRight" || e.key === "ArrowUp") {
      e.preventDefault();
      const next = Math.min(1.0, Math.round((mod.amount + 0.05) * 100) / 100);
      onAmountChange(mod.id, next);
    } else if (e.key === "ArrowLeft" || e.key === "ArrowDown") {
      e.preventDefault();
      const next = Math.max(0.0, Math.round((mod.amount - 0.05) * 100) / 100);
      onAmountChange(mod.id, next);
    } else if (e.key === "Home") {
      e.preventDefault();
      onAmountChange(mod.id, 0.0);
    } else if (e.key === "End") {
      e.preventDefault();
      onAmountChange(mod.id, 1.0);
    }
  };

  return (
    <div
      role="region"
      aria-label="Dynamic Effects Rack"
      className="grid grid-cols-5 gap-2.5 w-full h-full"
    >
      {modules.map((mod) => {
        const isModActive = mod.enabled && !disabled && !mod.isComingSoon;
        const defaultVal = DEFAULT_AMOUNTS[mod.id] ?? 0.5;

        return (
          <div
            key={mod.id}
            tabIndex={disabled || mod.isComingSoon ? -1 : 0}
            onKeyDown={(e) => handleKeyDown(e, mod)}
            className={`group bg-surface-card border rounded-lg p-3 flex flex-col justify-between h-full min-h-[115px] transition-all select-none relative outline-none overflow-hidden focus-visible:ring-1 focus-visible:ring-accent-cyan ${
              isModActive
                ? "border-accent-cyan/60 shadow-cyan-glow-sm bg-surface-card-hover"
                : "border-border-subtle hover:border-border-default"
            } ${disabled ? "opacity-50 pointer-events-none" : ""}`}
            title={`${mod.name}: ${mod.description}`}
          >
            {/* Header: Icon + Title + Switch */}
            <div className="flex items-center justify-between gap-1 mb-1">
              <div
                className="flex items-center gap-1.5 min-w-0 cursor-pointer"
                onClick={() => {
                  if (!mod.isComingSoon && !disabled) {
                    onToggleModule(mod.id);
                  }
                }}
              >
                <span
                  className={`transition-colors shrink-0 ${
                    isModActive ? "text-accent-cyan" : "text-text-muted"
                  }`}
                >
                  {getIcon(mod.id)}
                </span>
                <span
                  className={`text-[11px] font-semibold tracking-wide truncate ${
                    isModActive ? "text-text-primary" : "text-text-secondary"
                  }`}
                >
                  {getModuleName(mod.id, mod.name)}
                </span>
              </div>

              {/* Power Switch or Badge */}
              {mod.isComingSoon ? (
                <span
                  className="text-[9px] uppercase tracking-wider font-medium px-1.5 py-0.5 rounded bg-surface-elevated text-text-muted border border-border-default/60 shrink-0"
                  title={t.effectsRack.pitchSoonTooltip}
                >
                  {t.common.soon}
                </span>
              ) : (
                <button
                  type="button"
                  role="switch"
                  aria-checked={isModActive}
                  aria-label={`Toggle ${getModuleName(mod.id, mod.name)}`}
                  disabled={disabled}
                  onClick={(e) => {
                    e.stopPropagation();
                    onToggleModule(mod.id);
                  }}
                  className={`w-6 h-3.5 rounded-full transition-colors relative cursor-pointer shrink-0 focus:outline-none ${
                    isModActive ? "bg-accent-cyan" : "bg-border-default"
                  }`}
                >
                  <span
                    className={`absolute top-0.5 left-0.5 w-2.5 h-2.5 rounded-full bg-white transition-transform ${
                      isModActive ? "transform translate-x-2.5" : ""
                    }`}
                  />
                </button>
              )}
            </div>

            {/* Description Subtitle */}
            <div className="text-[10px] text-text-muted truncate mb-1.5 font-sans">
              {getModuleDescription(mod.id, mod.description)}
            </div>

            {/* Bottom Slider & Percentage Readout */}
            <div className="flex items-center gap-2 pt-1 min-w-0">
              {mod.isComingSoon ? (
                <div className="flex-1 min-w-0 flex items-center justify-between text-[10px] font-mono text-text-muted/60 px-1.5 py-1 bg-surface-base/40 rounded border border-border-subtle/40">
                  <span>Phase 2</span>
                  <span>0 st</span>
                </div>
              ) : (
                <>
                  <input
                    type="range"
                    role="slider"
                    aria-label={`${getModuleName(mod.id, mod.name)} intensity`}
                    aria-valuenow={Math.round(mod.amount * 100)}
                    aria-valuemin={0}
                    aria-valuemax={100}
                    min="0"
                    max="1"
                    step="0.01"
                    value={mod.amount}
                    disabled={!isModActive}
                    onDoubleClick={() => onAmountChange(mod.id, defaultVal)}
                    onChange={(e) =>
                      onAmountChange(mod.id, parseFloat(e.target.value))
                    }
                    className="flex-1 min-w-0 h-1.5 bg-border-subtle rounded appearance-none cursor-pointer accent-accent-cyan disabled:opacity-30 disabled:cursor-not-allowed"
                  />
                  <span
                    className={`text-[10px] font-mono w-7 shrink-0 text-right transition-colors ${
                      isModActive ? "text-text-primary" : "text-text-muted"
                    }`}
                  >
                    {Math.round(mod.amount * 100)}%
                  </span>
                </>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
};
