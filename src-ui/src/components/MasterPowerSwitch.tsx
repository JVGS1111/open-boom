import React from "react";

export interface MasterPowerSwitchProps {
  enabled: boolean;
  onToggle: () => void;
  disabled?: boolean;
  className?: string;
}

/**
 * Master Power Switch Component (UX-DR2, FR27).
 *
 * Dimensions: exactly 48px × 26px with full pill radius.
 * Active state: animated cyan LED glow effect (rgba(56, 189, 248, 0.35)).
 * Accessible with keyboard navigation (Space / Enter) and ARIA switch role.
 */
export const MasterPowerSwitch: React.FC<MasterPowerSwitchProps> = ({
  enabled,
  onToggle,
  disabled = false,
  className = "",
}) => {
  const handleKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>) => {
    if (disabled) return;
    if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      onToggle();
    }
  };

  return (
    <button
      type="button"
      role="switch"
      aria-checked={enabled}
      aria-disabled={disabled}
      aria-label={enabled ? "Master DSP Active" : "DSP Bypassed"}
      disabled={disabled}
      onClick={onToggle}
      onKeyDown={handleKeyDown}
      title={enabled ? "Master DSP Active (Click to Bypass)" : "DSP Bypassed (Click to Activate)"}
      className={`relative w-[48px] h-[26px] rounded-full transition-all duration-200 cursor-pointer outline-none focus-visible:ring-2 focus-visible:ring-accent-cyan focus-visible:ring-offset-2 focus-visible:ring-offset-surface-base select-none shrink-0 ${
        disabled
          ? "opacity-50 cursor-not-allowed bg-border-default"
          : enabled
          ? "bg-accent-cyan shadow-[0_0_12px_rgba(56,189,248,0.35)]"
          : "bg-border-default hover:bg-border-subtle"
      } ${className}`}
    >
      {/* Sliding Toggle Thumb */}
      <span
        className={`absolute top-[3px] left-[3px] w-[20px] h-[20px] rounded-full bg-white shadow-md transition-transform duration-200 ease-out pointer-events-none ${
          enabled ? "transform translate-x-[22px]" : "transform translate-x-0"
        }`}
      />
    </button>
  );
};

export default MasterPowerSwitch;
