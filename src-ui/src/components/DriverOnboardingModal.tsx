import React from "react";
import {
  Volume2,
  X,
  ShieldCheck,
  Sparkles,
  AlertTriangle,
  Download,
  ExternalLink,
  RefreshCw,
  CheckCircle2,
} from "lucide-react";
import { useI18n } from "../i18n";

export interface DriverOnboardingModalProps {
  isOpen: boolean;
  onClose: () => void;
  onInstallDriver?: () => Promise<void> | void;
  isInstalling?: boolean;
  installError?: string | null;
  onRetry?: () => void;
}

export const DriverOnboardingModal: React.FC<DriverOnboardingModalProps> = ({
  isOpen,
  onClose,
  onInstallDriver,
  isInstalling = false,
  installError = null,
  onRetry,
}) => {
  const { t } = useI18n();

  if (!isOpen) return null;

  const handleManualDownload = () => {
    // Open official VB-Cable website
    if (typeof window !== "undefined") {
      window.open("https://vb-audio.com/Cable/", "_blank");
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm animate-fade-in"
      role="dialog"
      aria-modal="true"
      aria-labelledby="driver-onboarding-title"
    >
      <div className="bg-[#16171b] border border-border-subtle/80 rounded-xl shadow-2xl w-[580px] max-w-[94vw] overflow-hidden flex flex-col text-text-primary animate-scale-up">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border-subtle/60 bg-[#121316]">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-lg bg-primary/10 border border-primary/30 flex items-center justify-center text-primary shadow-[0_0_12px_rgba(56,189,248,0.25)]">
              <Volume2 className="w-5 h-5" />
            </div>
            <div>
              <h2
                id="driver-onboarding-title"
                className="text-base font-semibold tracking-wide text-text-primary"
              >
                {t.driverOnboarding.modalTitle}
              </h2>
              <p className="text-xs text-text-muted">
                {t.driverOnboarding.modalSubtitle}
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/5 transition-colors focus:outline-none focus:ring-1 focus:ring-primary/50"
            aria-label={t.common.close}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Body */}
        <div className="p-6 flex flex-col gap-5 max-h-[75vh] overflow-y-auto">
          {/* Explanation */}
          <p className="text-sm text-text-secondary leading-relaxed">
            {t.driverOnboarding.description}
          </p>

          {/* Feature Highlights Grid */}
          <div className="grid grid-cols-1 gap-2.5">
            <div className="flex items-center gap-3 px-3.5 py-2.5 rounded-lg bg-[#1a1b20] border border-border-subtle/40">
              <CheckCircle2 className="w-4 h-4 text-primary shrink-0" />
              <span className="text-xs text-text-primary font-medium">
                {t.driverOnboarding.featureSurround}
              </span>
            </div>
            <div className="flex items-center gap-3 px-3.5 py-2.5 rounded-lg bg-[#1a1b20] border border-border-subtle/40">
              <ShieldCheck className="w-4 h-4 text-emerald-400 shrink-0" />
              <span className="text-xs text-text-primary font-medium">
                {t.driverOnboarding.featureNoEcho}
              </span>
            </div>
            <div className="flex items-center gap-3 px-3.5 py-2.5 rounded-lg bg-[#1a1b20] border border-border-subtle/40">
              <Sparkles className="w-4 h-4 text-amber-400 shrink-0" />
              <span className="text-xs text-text-primary font-medium">
                {t.driverOnboarding.featureLowLatency}
              </span>
            </div>
          </div>

          {/* Technical Why Box */}
          <div className="p-3.5 rounded-lg bg-surface-subtle/40 border border-border-subtle/40 text-xs text-text-muted leading-relaxed">
            <span className="font-semibold text-text-secondary block mb-1">
              {t.driverOnboarding.whyRequiredTitle}
            </span>
            {t.driverOnboarding.whyRequiredDesc}
          </div>

          {/* Error Alert Banner if failure occurred */}
          {installError && (
            <div
              className="p-3.5 rounded-lg bg-red-950/40 border border-red-500/40 flex items-start gap-3 text-red-200 text-xs animate-shake"
              role="alert"
            >
              <AlertTriangle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
              <div className="flex-1 flex flex-col gap-1.5">
                <span className="font-semibold text-red-300">
                  {t.driverOnboarding.errorTitle}
                </span>
                <span>{installError}</span>
                {onRetry && (
                  <button
                    onClick={onRetry}
                    className="self-start mt-1 px-2.5 py-1 rounded bg-red-800/50 hover:bg-red-700/60 text-red-100 font-medium text-[11px] flex items-center gap-1.5 transition-colors"
                  >
                    <RefreshCw className="w-3 h-3" />
                    {t.driverOnboarding.retryButton}
                  </button>
                )}
              </div>
            </div>
          )}

          {/* Primary Action Button */}
          <button
            onClick={() => onInstallDriver?.()}
            disabled={isInstalling}
            className={`w-full py-3 px-4 rounded-lg font-semibold text-sm flex items-center justify-center gap-2 transition-all shadow-lg ${
              isInstalling
                ? "bg-primary/50 text-white cursor-wait"
                : "bg-primary hover:bg-primary-hover text-[#0d0e12] hover:shadow-[0_0_20px_rgba(56,189,248,0.4)]"
            }`}
          >
            {isInstalling ? (
              <>
                <RefreshCw className="w-4 h-4 animate-spin" />
                {t.driverOnboarding.installingButton}
              </>
            ) : (
              <>
                <Download className="w-4 h-4" />
                {t.driverOnboarding.installButton}
              </>
            )}
          </button>

          {/* Manual Fallback Link */}
          <div className="flex items-center justify-between pt-1 border-t border-border-subtle/30 text-xs">
            <span className="text-text-muted">
              {t.driverOnboarding.manualDownloadDesc}
            </span>
            <button
              onClick={handleManualDownload}
              className="text-primary hover:underline flex items-center gap-1 text-xs font-medium"
            >
              <span>{t.driverOnboarding.manualDownloadLink}</span>
              <ExternalLink className="w-3 h-3" />
            </button>
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-3 bg-[#121316] border-t border-border-subtle/60 flex justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-lg text-xs font-medium text-text-muted hover:text-text-primary hover:bg-white/5 transition-colors"
          >
            {t.driverOnboarding.dismissLater}
          </button>
        </div>
      </div>
    </div>
  );
};
