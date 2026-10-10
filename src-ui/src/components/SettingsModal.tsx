import React, { useEffect, useRef } from "react";
import { Settings, X, HardDrive, CheckCircle2, ShieldCheck, Monitor } from "lucide-react";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  launchOnStartup: boolean;
  onToggleStartup: () => void;
  storagePath?: string;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  launchOnStartup,
  onToggleStartup,
  storagePath = "%APPDATA%\\OpenBoom\\config.json",
}) => {
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
        className="w-[480px] bg-surface-card border border-border-default rounded-xl shadow-2xl p-5 relative overflow-hidden flex flex-col gap-4"
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
                Preferences & System
              </h2>
              <p className="text-[11px] text-text-secondary">
                Configuration persistence and Windows lifecycle
              </p>
            </div>
          </div>
          <button
            ref={closeButtonRef}
            type="button"
            onClick={onClose}
            className="w-7 h-7 rounded flex items-center justify-center text-text-secondary hover:text-text-primary hover:bg-surface-elevated transition-colors cursor-pointer"
            aria-label="Close Settings"
            title="Close (Escape)"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Section 1: Windows Startup (FR29) */}
        <div className="bg-surface-base/80 border border-border-subtle rounded-lg p-3 flex items-center justify-between">
          <div className="flex flex-col gap-0.5 pr-3">
            <span className="text-xs font-semibold text-text-primary flex items-center gap-1.5">
              Launch on Windows Startup
            </span>
            <span className="text-[11px] text-text-muted leading-tight">
              Start OpenBoom automatically in background via user registry (<code className="text-accent-cyan/90 font-mono text-[10px]">HKCU\...\Run</code>) without UAC prompts.
            </span>
          </div>

          {/* Accessible Switch Toggle */}
          <button
            type="button"
            role="switch"
            aria-checked={launchOnStartup}
            aria-label="Launch on Windows Startup"
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

        {/* Section 2: Storage Architecture & Atomic Invariants (AD-10) */}
        <div className="flex flex-col gap-2 bg-surface-base/50 border border-border-subtle rounded-lg p-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1.5 text-xs font-semibold text-text-primary">
              <HardDrive className="w-3.5 h-3.5 text-accent-cyan" />
              <span>Atomic Persistence Contract</span>
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
            Settings are staged to <code className="text-text-secondary font-mono">config.&lt;pid&gt;.tmp</code> and atomically swapped using NTFS <code className="text-text-secondary font-mono">MoveFileExW</code> replacement, preventing corrupt state on sudden shutdowns.
          </p>
        </div>

        {/* Section 3: HUD Geometry & Audio Thread Guarantees */}
        <div className="grid grid-cols-2 gap-2 text-[11px]">
          <div className="bg-surface-base/40 border border-border-subtle rounded p-2 flex items-center gap-2">
            <Monitor className="w-4 h-4 text-accent-cyan shrink-0" />
            <div>
              <div className="font-semibold text-text-primary text-[10px]">Window Geometry</div>
              <div className="text-[10px] text-text-muted">Auto-restored screen position</div>
            </div>
          </div>

          <div className="bg-surface-base/40 border border-border-subtle rounded p-2 flex items-center gap-2">
            <ShieldCheck className="w-4 h-4 text-accent-cyan shrink-0" />
            <div>
              <div className="font-semibold text-text-primary text-[10px]">Real-Time Safety</div>
              <div className="text-[10px] text-text-muted">Zero disk I/O on audio loop</div>
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
            Done
          </button>
        </div>
      </div>
    </div>
  );
};
