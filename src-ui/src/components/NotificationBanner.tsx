import React, { useEffect } from "react";
import { AlertTriangle, Info, X } from "lucide-react";

export interface NotificationBannerProps {
  message: string | null;
  type?: "warning" | "info" | "error";
  onDismiss: () => void;
  autoDismissMs?: number;
}

export const NotificationBanner: React.FC<NotificationBannerProps> = ({
  message,
  type = "warning",
  onDismiss,
  autoDismissMs = 6000,
}) => {
  useEffect(() => {
    if (!message || autoDismissMs <= 0) return;
    const timer = setTimeout(() => {
      onDismiss();
    }, autoDismissMs);
    return () => clearTimeout(timer);
  }, [message, autoDismissMs, onDismiss]);

  if (!message) return null;

  const isWarning = type === "warning";
  const isError = type === "error";

  const borderColor = isError
    ? "border-meter-red/40"
    : isWarning
    ? "border-meter-amber/40"
    : "border-accent-cyan/40";

  const bgColor = isError
    ? "bg-meter-red/10"
    : isWarning
    ? "bg-meter-amber/10"
    : "bg-surface-card";

  const iconColor = isError
    ? "text-meter-red"
    : isWarning
    ? "text-meter-amber"
    : "text-accent-cyan";

  return (
    <div
      role="alert"
      aria-live="polite"
      className={`fixed top-16 left-1/2 -translate-x-1/2 z-50 max-w-md w-[90%] flex items-center justify-between gap-3 px-3.5 py-2 rounded-lg border shadow-xl backdrop-blur-md ${borderColor} ${bgColor} animate-in fade-in slide-in-from-top-2 duration-200`}
    >
      <div className="flex items-center gap-2.5 truncate">
        {isWarning || isError ? (
          <AlertTriangle className={`w-4 h-4 shrink-0 ${iconColor}`} />
        ) : (
          <Info className={`w-4 h-4 shrink-0 ${iconColor}`} />
        )}
        <span className="text-xs font-medium text-text-primary truncate">
          {message}
        </span>
      </div>
      <button
        type="button"
        onClick={onDismiss}
        aria-label="Dismiss notification"
        className="text-text-muted hover:text-text-primary p-0.5 rounded transition-colors shrink-0 cursor-pointer"
      >
        <X className="w-3.5 h-3.5" />
      </button>
    </div>
  );
};
