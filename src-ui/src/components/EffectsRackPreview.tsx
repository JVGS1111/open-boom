import React from "react";
import { Sparkles, Waves, Moon, Expand } from "lucide-react";

export interface EffectModule {
  id: number;
  name: string;
  description: string;
  enabled: boolean;
  amount: number;
}

interface EffectsRackPreviewProps {
  modules: EffectModule[];
  onToggleModule: (id: number) => void;
  onAmountChange: (id: number, val: number) => void;
}

export const EffectsRackPreview: React.FC<EffectsRackPreviewProps> = ({
  modules,
  onToggleModule,
  onAmountChange,
}) => {
  const getIcon = (id: number) => {
    switch (id) {
      case 0:
        return <Waves className="w-3.5 h-3.5" />;
      case 1:
        return <Sparkles className="w-3.5 h-3.5" />;
      case 2:
        return <Moon className="w-3.5 h-3.5" />;
      case 3:
      default:
        return <Expand className="w-3.5 h-3.5" />;
    }
  };

  return (
    <div className="grid grid-cols-2 gap-2 h-full">
      {modules.map((mod) => (
        <div
          key={mod.id}
          className={`bg-surface-card border rounded-lg p-2.5 flex flex-col justify-between transition-colors ${
            mod.enabled
              ? "border-accent-cyan/40 shadow-cyan-glow-sm bg-surface-card-hover"
              : "border-border-subtle"
          }`}
        >
          <div className="flex items-center justify-between mb-1">
            <div className="flex items-center gap-1.5">
              <span
                className={`${
                  mod.enabled ? "text-accent-cyan" : "text-text-muted"
                }`}
              >
                {getIcon(mod.id)}
              </span>
              <span className="text-xs font-semibold text-text-primary tracking-wide">
                {mod.name}
              </span>
            </div>

            {/* Toggle Button */}
            <button
              type="button"
              onClick={() => onToggleModule(mod.id)}
              className={`w-7 h-4 rounded-full transition-colors relative cursor-pointer ${
                mod.enabled ? "bg-accent-cyan" : "bg-border-default"
              }`}
            >
              <span
                className={`absolute top-0.5 left-0.5 w-3 h-3 rounded-full bg-white transition-transform ${
                  mod.enabled ? "transform translate-x-3" : ""
                }`}
              />
            </button>
          </div>

          <div className="text-[10px] text-text-secondary truncate mb-2">
            {mod.description}
          </div>

          {/* Amount Slider */}
          <div className="flex items-center gap-2">
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={mod.amount}
              disabled={!mod.enabled}
              onChange={(e) => onAmountChange(mod.id, parseFloat(e.target.value))}
              className="flex-1 h-1 bg-border-subtle rounded appearance-none cursor-pointer accent-accent-cyan disabled:opacity-40"
            />
            <span className="text-[10px] font-mono text-text-muted w-7 text-right">
              {Math.round(mod.amount * 100)}%
            </span>
          </div>
        </div>
      ))}
    </div>
  );
};
