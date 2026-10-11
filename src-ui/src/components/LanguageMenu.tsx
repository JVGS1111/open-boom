import React, { useState, useRef, useEffect } from "react";
import { Globe, Check, ChevronDown } from "lucide-react";
import { useI18n, LANGUAGE_OPTIONS, LanguageOption } from "../i18n";

export interface LanguageMenuProps {
  className?: string;
}

export const LanguageMenu: React.FC<LanguageMenuProps> = ({ className = "" }) => {
  const { language, effectiveLocale, setLanguage, t } = useI18n();
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // Close on outside click
  useEffect(() => {
    if (!isOpen) return;
    const handleOutsideClick = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setIsOpen(false);
      }
    };
    document.addEventListener("mousedown", handleOutsideClick);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handleOutsideClick);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen]);

  const activeBadge =
    language === "system"
      ? effectiveLocale === "pt-BR"
        ? "PT"
        : effectiveLocale === "es"
        ? "ES"
        : "EN"
      : language === "pt-BR"
      ? "PT"
      : language === "es"
      ? "ES"
      : "EN";

  const handleSelect = (langId: LanguageOption) => {
    setLanguage(langId);
    setIsOpen(false);
  };

  return (
    <div ref={containerRef} className={`relative inline-block ${className}`}>
      {/* Menu Trigger Button */}
      <button
        type="button"
        onClick={() => setIsOpen((prev) => !prev)}
        className={`h-8 px-2 rounded flex items-center gap-1.5 transition-all cursor-pointer ${
          isOpen
            ? "text-accent-cyan bg-accent-cyan/15 border border-accent-cyan/40 shadow-cyan-glow-sm"
            : "text-text-secondary hover:text-text-primary hover:bg-surface-card border border-transparent"
        }`}
        title={t.titlebar.languageTooltip}
        aria-label={t.titlebar.languageTooltip}
        aria-haspopup="true"
        aria-expanded={isOpen}
      >
        <Globe className="w-3.5 h-3.5 text-accent-cyan" />
        <span className="text-[10px] font-mono font-bold tracking-wider text-text-primary">
          {activeBadge}
        </span>
        <ChevronDown className="w-3 h-3 text-text-muted opacity-70" />
      </button>

      {/* Popover Dropdown Menu */}
      {isOpen && (
        <div
          role="menu"
          aria-label={t.titlebar.languageTooltip}
          className="absolute right-0 top-full mt-1 w-52 bg-surface-card border border-border-default rounded-xl shadow-2xl p-1.5 z-50 flex flex-col gap-0.5 animate-in fade-in duration-150 backdrop-blur-md"
        >
          <div className="px-2 py-1 text-[10px] font-semibold text-text-muted uppercase tracking-wider border-b border-border-subtle/60 mb-0.5 flex items-center justify-between">
            <span>{t.settings.languageTitle}</span>
            <span className="text-[9px] font-mono text-accent-cyan">{effectiveLocale}</span>
          </div>

          {LANGUAGE_OPTIONS.map((opt) => {
            const isSelected = language === opt.id;
            return (
              <button
                key={opt.id}
                type="button"
                role="menuitem"
                onClick={() => handleSelect(opt.id)}
                className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition-colors cursor-pointer text-left ${
                  isSelected
                    ? "bg-accent-cyan/15 text-accent-cyan font-medium border border-accent-cyan/30"
                    : "text-text-secondary hover:text-text-primary hover:bg-surface-elevated border border-transparent"
                }`}
              >
                <div className="flex items-center gap-2">
                  <span className="w-6 text-[10px] font-mono font-bold text-center px-1 py-0.5 bg-surface-base rounded border border-border-subtle text-text-primary">
                    {opt.badge}
                  </span>
                  <div className="flex flex-col">
                    <span className="leading-tight">{opt.nativeName}</span>
                    {opt.id === "system" && (
                      <span className="text-[10px] text-text-muted leading-tight">
                        ({effectiveLocale})
                      </span>
                    )}
                  </div>
                </div>
                {isSelected && <Check className="w-3.5 h-3.5 text-accent-cyan shrink-0" />}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
};
