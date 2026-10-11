import React, { useEffect, useRef } from "react";
import { Settings, X, HardDrive, CheckCircle2, ShieldCheck, Monitor, AlertTriangle, Globe } from "lucide-react";
import { useI18n, LANGUAGE_OPTIONS } from "../i18n";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  launchOnStartup: boolean;
  onToggleStartup: () => void;
  bufferGuardEnabled?: boolean;
  onToggleBufferGuard?: () => void;
  storagePath?: string;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  launchOnStartup,
  onToggleStartup,
  bufferGuardEnabled = true,
  onToggleBufferGuard,
  storagePath = "%APPDATA%\\OpenBoom\\config.json",
}) => {
  const { t, language, setLanguage } = useI18n();
  const modalRef = useRef<HTMLDivElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  // Keyboard accessibility: Escape to dismiss, focus trap handling (UX-DR11)
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    closeButtonRef.current?.focus();

    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-modal-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm select-none"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        ref={modalRef}
        className="w-[490px] max-h-[90vh] bg-surface-card border border-border-default rounded-xl shadow-2xl p-5 relative overflow-y-auto flex flex-col gap-3.5"
      >
        {/* Header */}
        <div className="flex items-center justify-between pb-3 border-b border-border-subtle">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-surface-elevated border border-border-default flex items-center justify-center text-accent-cyan shadow-cyan-glow-sm">
              <Settings className="w-4 h-4" />
            </div>
            <div>
              <h2
                id="settings-modal-title"
                className="font-display font-bold text-sm tracking-wide text-text-primary"
              >
                {t.settings.modalTitle}
              </h2>
              <p className="text-[11px] text-text-secondary">
                {t.settings.modalSubtitle}
              </p>
            </div>
          </div>
          <button
            ref={closeButtonRef}
            type="button"
            onClick={onClose}
            className="w-7 h-7 rounded flex items-center justify-center text-text-secondary hover:text-text-primary hover:bg-surface-elevated transition-colors cursor-pointer"
            aria-label={t.common.close}
            title={`${t.common.close} (Escape)`}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Section 1: Language & Interface */}
        <div className="bg-surface-base/80 border border-border-subtle rounded-lg p-3 flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <div className="flex flex-col gap-0.5 pr-3">
              <span className="text-xs font-semibold text-text-primary flex items-center gap-1.5">
                <Globe className="w-3.5 h-3.5 text-accent-cyan" />
                {t.settings.languageTitle}
              </span>
              <span className="text-[11px] text-text-muted leading-tight">
                {t.settings.languageSubtitle}
              </span>
            </div>
          </div>

          <div className="grid grid-cols-4 gap-1.5 pt-1">
            {LANGUAGE_OPTIONS.map((opt) => {
              const isSelected = language === opt.id;
              return (
                <button
                  key={opt.id}
                  type="button"
                  onClick={() => setLanguage(opt.id)}
                  className={`px-2 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer flex flex-col items-center gap-0.5 ${
                    isSelected
                      ? "bg-accent-cyan/15 text-accent-cyan border border-accent-cyan/50 shadow-cyan-glow-sm"
                      : "bg-surface-elevated hover:bg-surface-card border border-border-subtle text-text-secondary hover:text-text-primary"
                  }`}
                >
                  <span className="text-[10px] font-mono font-bold">{opt.badge}</span>
                  <span className="text-[11px] truncate max-w-full">{opt.nativeName}</span>
                </button>
              );
            })}
          </div>
        </div>

        {/* Section 2: Windows Startup (FR29) */}
        <div className="bg-surface-base/80 border border-border-subtle rounded-lg p-3 flex items-center justify-between">
          <div className="flex flex-col gap-0.5 pr-3">
            <span className="text-xs font-semibold text-text-primary flex items-center gap-1.5">
              {t.settings.startupTitle}
            </span>
            <span className="text-[11px] text-text-muted leading-tight">
              {t.settings.startupDesc}
            </span>
          </div>

          {/* Accessible Switch Toggle */}
          <button
            type="button"
            role="switch"
            aria-checked={launchOnStartup}
            aria-label={t.settings.startupTitle}
            onClick={onToggleStartup}
            className={`w-11 h-6 shrink-0 rounded-full p-0.5 transition-colors duration-200 ease-in-out cursor-pointer relative ${
              launchOnStartup
                ? "bg-accent-cyan shadow-cyan-glow-sm"
                : "bg-surface-elevated border border-border-subtle"
            }`}
          >
            <div
              className={`w-5 h-5 rounded-full bg-white transition-transform duration-200 ease-in-out shadow-md ${
                launchOnStartup ? "translate-x-5" : "translate-x-0"
              }`}
            />
          </button>
        </div>

        {/* Section 3: Audio Buffer Stability & Acoustic Guard */}
        <div className="bg-surface-base/80 border border-border-subtle rounded-lg p-3 flex flex-col gap-2.5">
          <div className="flex items-center justify-between">
            <div className="flex flex-col gap-0.5 pr-3">
              <span className="text-xs font-semibold text-text-primary flex items-center gap-1.5">
                <ShieldCheck className="w-3.5 h-3.5 text-accent-cyan" />
                {t.settings.bufferGuardTitle}
              </span>
              <span className="text-[11px] text-text-muted leading-tight">
                {t.settings.bufferGuardDesc}
              </span>
            </div>

            {/* Switch Toggle */}
            <button
              type="button"
              role="switch"
              aria-checked={bufferGuardEnabled}
              aria-label={t.settings.bufferGuardTitle}
              onClick={onToggleBufferGuard}
              className={`w-11 h-6 shrink-0 rounded-full p-0.5 transition-colors duration-200 ease-in-out cursor-pointer relative ${
                bufferGuardEnabled
                  ? "bg-accent-cyan shadow-cyan-glow-sm"
                  : "bg-surface-elevated border border-border-subtle"
              }`}
            >
              <div
                className={`w-5 h-5 rounded-full bg-white transition-transform duration-200 ease-in-out shadow-md ${
                  bufferGuardEnabled ? "translate-x-5" : "translate-x-0"
                }`}
              />
            </button>
          </div>

          {/* Conditional Alert Banner */}
          {!bufferGuardEnabled ? (
            <div className="bg-status-offline/10 border border-status-offline/40 rounded-md p-2.5 flex items-start gap-2 text-status-offline animate-in fade-in duration-200">
              <AlertTriangle className="w-4 h-4 shrink-0 mt-0.5 text-status-offline animate-pulse" />
              <div className="text-[11px] leading-tight">
                <span className="font-semibold block text-red-400 mb-0.5">
                  {t.settings.bufferGuardWarningTitle}
                </span>
                {t.settings.bufferGuardWarningDesc}
              </div>
            </div>
          ) : (
            <div className="flex items-center gap-1.5 text-[10px] text-accent-cyan font-mono bg-accent-cyan/10 border border-accent-cyan/20 px-2 py-1 rounded">
              <CheckCircle2 className="w-3 h-3 text-status-online" />
              <span>{t.settings.bufferGuardActive}</span>
            </div>
          )}
        </div>

        {/* Section 4: Storage Architecture & Atomic Invariants (AD-10) */}
        <div className="flex flex-col gap-2 bg-surface-base/50 border border-border-subtle rounded-lg p-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1.5 text-xs font-semibold text-text-primary">
              <HardDrive className="w-3.5 h-3.5 text-accent-cyan" />
              <span>{t.settings.atomicTitle}</span>
            </div>
            <span className="flex items-center gap-1 text-[10px] font-mono text-status-online bg-status-online/10 px-1.5 py-0.5 rounded border border-status-online/20">
              <CheckCircle2 className="w-3 h-3" />
              NTFS Atomic Rename
            </span>
          </div>

          <div className="text-[11px] font-mono bg-surface-overlay border border-border-subtle px-2 py-1.5 rounded text-text-secondary truncate select-all">
            {storagePath}
          </div>

          <p className="text-[10px] text-text-muted leading-relaxed">
            {t.settings.atomicDesc}
          </p>
        </div>

        {/* Section 5: HUD Geometry & Audio Thread Guarantees */}
        <div className="grid grid-cols-2 gap-2 text-[11px]">
          <div className="bg-surface-base/40 border border-border-subtle rounded p-2 flex items-center gap-2">
            <Monitor className="w-4 h-4 text-accent-cyan shrink-0" />
            <div>
              <div className="font-semibold text-text-primary text-[10px]">{t.settings.windowGeometry}</div>
              <div className="text-[10px] text-text-muted">{t.settings.windowGeometryDesc}</div>
            </div>
          </div>

          <div className="bg-surface-base/40 border border-border-subtle rounded p-2 flex items-center gap-2">
            <ShieldCheck className="w-4 h-4 text-accent-cyan shrink-0" />
            <div>
              <div className="font-semibold text-text-primary text-[10px]">{t.settings.realTimeSafety}</div>
              <div className="text-[10px] text-text-muted">{t.settings.realTimeSafetyDesc}</div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-end pt-2 border-t border-border-subtle">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-1.5 rounded-lg bg-surface-elevated hover:bg-surface-card-hover border border-border-default text-xs font-medium text-text-primary transition-colors cursor-pointer"
          >
            {t.common.done}
          </button>
        </div>
      </div>
    </div>
  );
};
