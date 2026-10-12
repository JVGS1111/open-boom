import React, { useEffect, useRef, useState } from "react";
import {
  Settings,
  X,
  HardDrive,
  CheckCircle2,
  ShieldCheck,
  Monitor,
  AlertTriangle,
  Globe,
  ArrowUpCircle,
  RefreshCw,
  Sparkles,
  Download,
  RotateCw,
  Radio,
} from "lucide-react";
import { useI18n, LANGUAGE_OPTIONS } from "../i18n";

export interface UpdateInfo {
  isAvailable: boolean;
  currentVersion: string;
  latestVersion: string;
  releaseNotes?: string;
  publishedAt?: string;
  downloadUrl?: string;
}

export type SettingsTab = "general" | "updates";
export type AudioBridgeMode = "auto" | "virtual" | "loopback";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  launchOnStartup: boolean;
  onToggleStartup: () => void;
  bufferGuardEnabled?: boolean;
  onToggleBufferGuard?: () => void;
  audioBridgeMode?: AudioBridgeMode;
  usingVirtualBridge?: boolean;
  isSwitchingBridge?: boolean;
  onChangeAudioBridgeMode?: (mode: AudioBridgeMode) => void;
  storagePath?: string;
  initialTab?: SettingsTab;
  // Updater properties
  checkUpdatesOnStartup?: boolean;
  onToggleCheckUpdatesOnStartup?: () => void;
  autoInstallUpdates?: boolean;
  onToggleAutoInstallUpdates?: () => void;
  updateInfo?: UpdateInfo | null;
  isCheckingUpdates?: boolean;
  isDownloadingUpdate?: boolean;
  isUpdateReadyToRestart?: boolean;
  updateError?: string | null;
  onCheckForUpdates?: () => void;
  onInstallUpdate?: () => void;
  onRestartApplication?: () => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  onClose,
  launchOnStartup,
  onToggleStartup,
  bufferGuardEnabled = true,
  onToggleBufferGuard,
  audioBridgeMode = "auto",
  usingVirtualBridge = false,
  isSwitchingBridge = false,
  onChangeAudioBridgeMode,
  storagePath = "%APPDATA%\\OpenBoom\\config.json",
  initialTab = "general",
  checkUpdatesOnStartup = true,
  onToggleCheckUpdatesOnStartup,
  autoInstallUpdates = false,
  onToggleAutoInstallUpdates,
  updateInfo,
  isCheckingUpdates = false,
  isDownloadingUpdate = false,
  isUpdateReadyToRestart = false,
  updateError,
  onCheckForUpdates,
  onInstallUpdate,
  onRestartApplication,
}) => {
  const { t, language, setLanguage } = useI18n();
  const [activeTab, setActiveTab] = useState<SettingsTab>(initialTab);
  const modalRef = useRef<HTMLDivElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  // Synchronize initialTab when modal opens or initialTab prop changes
  useEffect(() => {
    if (isOpen) {
      setActiveTab(initialTab);
    }
  }, [isOpen, initialTab]);

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
        className="w-[520px] max-h-[90vh] bg-surface-card border border-border-default rounded-xl shadow-2xl p-5 relative overflow-y-auto flex flex-col gap-3.5"
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

        {/* Tab Navigation: General Preferences vs Updates */}
        <div className="flex items-center gap-2 border-b border-border-subtle pb-2">
          <button
            type="button"
            onClick={() => setActiveTab("general")}
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer flex items-center gap-1.5 ${
              activeTab === "general"
                ? "bg-accent-cyan/15 text-accent-cyan border border-accent-cyan/40 shadow-cyan-glow-sm"
                : "text-text-secondary hover:text-text-primary hover:bg-surface-elevated border border-transparent"
            }`}
          >
            <Settings className="w-3.5 h-3.5" />
            <span>{t.settings.generalTab}</span>
          </button>

          <button
            type="button"
            onClick={() => setActiveTab("updates")}
            className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer flex items-center gap-1.5 relative ${
              activeTab === "updates"
                ? "bg-accent-cyan/15 text-accent-cyan border border-accent-cyan/40 shadow-cyan-glow-sm"
                : "text-text-secondary hover:text-text-primary hover:bg-surface-elevated border border-transparent"
            }`}
          >
            <ArrowUpCircle className="w-3.5 h-3.5" />
            <span>{t.settings.updatesTitle}</span>
            {updateInfo?.isAvailable && (
              <span className="w-2 h-2 rounded-full bg-amber-400 animate-ping absolute -top-0.5 -right-0.5" />
            )}
          </button>
        </div>

        {/* TAB 1: GENERAL SETTINGS */}
        {activeTab === "general" && (
          <div className="flex flex-col gap-3.5 animate-in fade-in duration-150">
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

            {/* Section 2.5: Audio Capture Bridge Mode (Driver Boom Audio vs. Loopback Emulation) */}
            <div className="bg-surface-base/80 border border-border-subtle rounded-lg p-3 flex flex-col gap-2.5">
              <div className="flex flex-col gap-0.5">
                <span className="text-xs font-semibold text-text-primary flex items-center gap-1.5">
                  <Radio className="w-3.5 h-3.5 text-accent-cyan" />
                  {t.settings.bridgeModeTitle}
                </span>
                <span className="text-[11px] text-text-muted leading-tight">
                  {t.settings.bridgeModeDesc}
                </span>
              </div>

              <select
                data-testid="audio-bridge-mode-select"
                value={audioBridgeMode}
                disabled={isSwitchingBridge}
                aria-label={t.settings.bridgeModeSelectAria}
                onChange={(e) => {
                  const val = e.target.value as AudioBridgeMode;
                  if (val === audioBridgeMode) return;
                  if (onChangeAudioBridgeMode) {
                    onChangeAudioBridgeMode(val);
                  }
                }}
                className="w-full px-2.5 py-2 rounded-lg bg-surface-elevated border border-border-default text-xs font-medium text-text-primary focus:outline-none focus:border-accent-cyan focus:ring-1 focus:ring-accent-cyan/40 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <option value="auto" className="bg-surface-card text-text-primary">
                  {t.settings.bridgeModeOptionAuto}
                </option>
                <option value="virtual" className="bg-surface-card text-text-primary">
                  {t.settings.bridgeModeOptionVirtual}
                </option>
                <option value="loopback" className="bg-surface-card text-text-primary">
                  {t.settings.bridgeModeOptionLoopback}
                </option>
              </select>

              <div
                data-testid="audio-bridge-active-badge"
                className={`flex items-center gap-1.5 text-[10px] font-mono px-2 py-1 rounded border ${
                  usingVirtualBridge && audioBridgeMode !== "loopback"
                    ? "text-status-online bg-status-online/10 border-status-online/25"
                    : "text-status-warning bg-status-warning/10 border-status-warning/25"
                }`}
              >
                <CheckCircle2
                  className={`w-3 h-3 shrink-0 ${
                    usingVirtualBridge && audioBridgeMode !== "loopback"
                      ? "text-status-online"
                      : "text-status-warning"
                  }`}
                />
                <span>
                  {usingVirtualBridge && audioBridgeMode !== "loopback"
                    ? t.settings.bridgeModeActiveVirtual
                    : t.settings.bridgeModeActiveLoopback}
                </span>
              </div>

              {(audioBridgeMode === "loopback" || !usingVirtualBridge) && (
                <div
                  data-testid="audio-bridge-loopback-warning"
                  className="bg-status-warning/10 border border-status-warning/40 rounded-md p-2.5 flex items-start gap-2 text-status-warning animate-in fade-in duration-200"
                >
                  <AlertTriangle className="w-4 h-4 shrink-0 mt-0.5 text-status-warning" />
                  <div className="text-[11px] leading-tight">
                    <span className="font-semibold block text-amber-400 mb-0.5">
                      {t.settings.bridgeModeLoopbackWarningTitle}
                    </span>
                    {t.settings.bridgeModeLoopbackWarningDesc}
                  </div>
                </div>
              )}

              <div className="flex items-center gap-1.5 text-[10px] text-text-muted">
                <RotateCw className="w-3 h-3 text-accent-cyan shrink-0" />
                <span>{t.settings.bridgeModeRestartNotice}</span>
              </div>
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

            {/* Section 4: Storage Architecture & Atomic Invariants */}
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
          </div>
        )}

        {/* TAB 2: APPLICATION UPDATES */}
        {activeTab === "updates" && (
          <div className="flex flex-col gap-3.5 animate-in fade-in duration-150">
            {/* Version & Status Banner */}
            <div className="bg-surface-base/80 border border-border-subtle rounded-lg p-3.5 flex flex-col gap-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="w-8 h-8 rounded-lg bg-surface-elevated border border-border-default flex items-center justify-center text-amber-400">
                    <ArrowUpCircle className="w-4 h-4" />
                  </div>
                  <div>
                    <h3 className="text-xs font-semibold text-text-primary">
                      {t.settings.updatesTitle}
                    </h3>
                    <p className="text-[11px] text-text-muted">
                      {t.settings.updatesSubtitle}
                    </p>
                  </div>
                </div>

                <div className="flex items-center gap-2 font-mono text-[11px]">
                  <span className="px-2 py-0.5 rounded bg-surface-elevated border border-border-subtle text-text-secondary">
                    {t.settings.currentVersionLabel}: <strong className="text-text-primary">{updateInfo?.currentVersion || "0.5.0"}</strong>
                  </span>
                  {updateInfo?.latestVersion && (
                    <span className={`px-2 py-0.5 rounded border ${
                      updateInfo.isAvailable
                        ? "bg-amber-400/10 border-amber-400/30 text-amber-400 font-bold"
                        : "bg-surface-elevated border-border-subtle text-text-secondary"
                    }`}>
                      {t.settings.latestVersionLabel}: <strong>{updateInfo.latestVersion}</strong>
                    </span>
                  )}
                </div>
              </div>

              {/* Status Message */}
              {updateError ? (
                <div className="bg-status-offline/10 border border-status-offline/30 rounded p-2 text-status-offline text-xs flex items-center gap-2">
                  <AlertTriangle className="w-4 h-4 shrink-0" />
                  <span>{updateError}</span>
                </div>
              ) : isUpdateReadyToRestart ? (
                <div className="bg-status-online/15 border border-status-online/40 rounded p-2.5 text-status-online text-xs flex items-center gap-2">
                  <CheckCircle2 className="w-4 h-4 shrink-0 text-status-online" />
                  <span className="font-semibold">{t.settings.updateReadyMessage}</span>
                </div>
              ) : updateInfo?.isAvailable ? (
                <div className="bg-amber-400/15 border border-amber-400/40 rounded p-2.5 text-amber-400 text-xs flex items-center gap-2">
                  <Sparkles className="w-4 h-4 shrink-0" />
                  <span className="font-semibold">
                    {t.settings.updateAvailableMessage} (v{updateInfo.latestVersion})
                    {updateInfo.publishedAt && ` â€¢ ${t.settings.publishedOn} ${new Date(updateInfo.publishedAt).toLocaleDateString()}`}
                  </span>
                </div>
              ) : (
                <div className="bg-surface-elevated/70 border border-border-subtle rounded p-2 text-text-secondary text-xs flex items-center gap-2">
                  <CheckCircle2 className="w-4 h-4 shrink-0 text-accent-cyan" />
                  <span>{t.settings.upToDateMessage}</span>
                </div>
              )}

              {/* Release Notes Preview */}
              {updateInfo?.isAvailable && updateInfo.releaseNotes && (
                <div className="flex flex-col gap-1.5 bg-surface-elevated/50 border border-border-subtle rounded p-2.5">
                  <span className="text-[10px] font-bold text-text-muted uppercase tracking-wider font-mono">
                    {t.settings.releaseNotesTitle}
                  </span>
                  <div className="text-[11px] text-text-secondary whitespace-pre-wrap max-h-32 overflow-y-auto leading-relaxed select-text font-sans">
                    {updateInfo.releaseNotes}
                  </div>
                </div>
              )}

              {/* Action Button Area */}
              <div className="flex items-center justify-end gap-2 pt-1">
                {isUpdateReadyToRestart ? (
                  <button
                    type="button"
                    onClick={onRestartApplication}
                    className="px-4 py-2 rounded-lg bg-emerald-500 hover:bg-emerald-400 text-black font-semibold text-xs transition-all shadow-[0_0_12px_rgba(16,185,129,0.35)] flex items-center gap-1.5 cursor-pointer"
                  >
                    <RotateCw className="w-3.5 h-3.5" />
                    <span>{t.settings.restartApplicationButton}</span>
                  </button>
                ) : updateInfo?.isAvailable ? (
                  <button
                    type="button"
                    onClick={onInstallUpdate}
                    disabled={isDownloadingUpdate}
                    className="px-4 py-2 rounded-lg bg-amber-400 hover:bg-amber-300 text-black font-semibold text-xs transition-all shadow-[0_0_12px_rgba(251,191,36,0.35)] flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                  >
                    {isDownloadingUpdate ? (
                      <>
                        <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                        <span>{t.settings.downloadingUpdate}</span>
                      </>
                    ) : (
                      <>
                        <Download className="w-3.5 h-3.5" />
                        <span>{t.settings.downloadAndInstallButton}</span>
                      </>
                    )}
                  </button>
                ) : (
                  <button
                    type="button"
                    onClick={onCheckForUpdates}
                    disabled={isCheckingUpdates}
                    className="px-3.5 py-1.5 rounded-lg bg-surface-elevated hover:bg-surface-card border border-border-default text-text-primary text-xs font-medium transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                  >
                    <RefreshCw className={`w-3.5 h-3.5 text-accent-cyan ${isCheckingUpdates ? "animate-spin" : ""}`} />
                    <span>{isCheckingUpdates ? t.settings.checkingForUpdates : t.settings.checkForUpdatesButton}</span>
                  </button>
                )}
              </div>
            </div>

            {/* Toggle 1: Check for updates on startup */}
            <div className="bg-surface-base/80 border border-border-subtle rounded-lg p-3 flex items-center justify-between">
              <div className="flex flex-col gap-0.5 pr-3">
                <span className="text-xs font-semibold text-text-primary">
                  {t.settings.checkUpdatesOnStartupTitle}
                </span>
                <span className="text-[11px] text-text-muted leading-tight">
                  {t.settings.checkUpdatesOnStartupDesc}
                </span>
              </div>

              <button
                type="button"
                role="switch"
                aria-checked={checkUpdatesOnStartup}
                aria-label={t.settings.checkUpdatesOnStartupTitle}
                onClick={onToggleCheckUpdatesOnStartup}
                className={`w-11 h-6 shrink-0 rounded-full p-0.5 transition-colors duration-200 ease-in-out cursor-pointer relative ${
                  checkUpdatesOnStartup
                    ? "bg-accent-cyan shadow-cyan-glow-sm"
                    : "bg-surface-elevated border border-border-subtle"
                }`}
              >
                <div
                  className={`w-5 h-5 rounded-full bg-white transition-transform duration-200 ease-in-out shadow-md ${
                    checkUpdatesOnStartup ? "translate-x-5" : "translate-x-0"
                  }`}
                />
              </button>
            </div>

            {/* Toggle 2: Automatically install updates when available */}
            <div className="bg-surface-base/80 border border-border-subtle rounded-lg p-3 flex items-center justify-between">
              <div className="flex flex-col gap-0.5 pr-3">
                <span className="text-xs font-semibold text-text-primary">
                  {t.settings.autoInstallUpdatesTitle}
                </span>
                <span className="text-[11px] text-text-muted leading-tight">
                  {t.settings.autoInstallUpdatesDesc}
                </span>
              </div>

              <button
                type="button"
                role="switch"
                aria-checked={autoInstallUpdates}
                aria-label={t.settings.autoInstallUpdatesTitle}
                onClick={onToggleAutoInstallUpdates}
                className={`w-11 h-6 shrink-0 rounded-full p-0.5 transition-colors duration-200 ease-in-out cursor-pointer relative ${
                  autoInstallUpdates
                    ? "bg-accent-cyan shadow-cyan-glow-sm"
                    : "bg-surface-elevated border border-border-subtle"
                }`}
              >
                <div
                  className={`w-5 h-5 rounded-full bg-white transition-transform duration-200 ease-in-out shadow-md ${
                    autoInstallUpdates ? "translate-x-5" : "translate-x-0"
                  }`}
                />
              </button>
            </div>
          </div>
        )}

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
