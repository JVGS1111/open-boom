import React, { createContext, useContext, useState, useEffect, useMemo } from "react";
import { SupportedLocale, LanguageOption, TranslationDictionary } from "./types";
import { en } from "./locales/en";
import { ptBR } from "./locales/pt-BR";
import { es } from "./locales/es";

export * from "./types";

export const TRANSLATIONS: Record<SupportedLocale, TranslationDictionary> = {
  "pt-BR": ptBR,
  en: en,
  es: es,
};

export interface LanguageInfo {
  id: LanguageOption;
  localeCode?: SupportedLocale;
  label: string;
  nativeName: string;
  badge: string;
}

export const LANGUAGE_OPTIONS: LanguageInfo[] = [
  {
    id: "system",
    label: "System Default",
    nativeName: "Padrão do Sistema",
    badge: "SYS",
  },
  {
    id: "pt-BR",
    localeCode: "pt-BR",
    label: "Portuguese (Brazil)",
    nativeName: "Português (Brasil)",
    badge: "PT",
  },
  {
    id: "en",
    localeCode: "en",
    label: "English",
    nativeName: "English",
    badge: "EN",
  },
  {
    id: "es",
    localeCode: "es",
    label: "Spanish",
    nativeName: "Español",
    badge: "ES",
  },
];

/**
 * Detects the Windows host / browser environment system locale.
 * Defaults to "pt-BR" if language starts with "pt", "es" if "es", otherwise "en".
 */
export function getSystemLocale(): SupportedLocale {
  if (typeof navigator !== "undefined" && navigator.language) {
    const lang = navigator.language.toLowerCase();
    if (lang.startsWith("pt")) {
      return "pt-BR";
    }
    if (lang.startsWith("es")) {
      return "es";
    }
  }
  return "en";
}

/**
 * Resolves the effective locale given a language option.
 */
export function resolveEffectiveLocale(option: LanguageOption): SupportedLocale {
  if (option === "system" || !option) {
    return getSystemLocale();
  }
  if (option === "pt-BR" || option === "en" || option === "es") {
    return option;
  }
  return "en";
}

interface I18nContextType {
  language: LanguageOption;
  effectiveLocale: SupportedLocale;
  setLanguage: (lang: LanguageOption) => void;
  t: TranslationDictionary;
}

const I18nContext = createContext<I18nContextType>({
  language: "system",
  effectiveLocale: "en",
  setLanguage: () => {},
  t: en,
});

export interface I18nProviderProps {
  initialLanguage?: LanguageOption;
  onLanguageChange?: (lang: LanguageOption) => void;
  children: React.ReactNode;
}

export const I18nProvider: React.FC<I18nProviderProps> = ({
  initialLanguage = "system",
  onLanguageChange,
  children,
}) => {
  const [language, setLanguageState] = useState<LanguageOption>(initialLanguage);

  useEffect(() => {
    if (initialLanguage && initialLanguage !== language) {
      setLanguageState(initialLanguage);
    }
  }, [initialLanguage]);

  const effectiveLocale = useMemo(() => resolveEffectiveLocale(language), [language]);

  const setLanguage = (lang: LanguageOption) => {
    setLanguageState(lang);
    if (onLanguageChange) {
      onLanguageChange(lang);
    }
  };

  const t = useMemo(() => TRANSLATIONS[effectiveLocale] || en, [effectiveLocale]);

  const value = useMemo(
    () => ({
      language,
      effectiveLocale,
      setLanguage,
      t,
    }),
    [language, effectiveLocale, t]
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
};

export const useI18n = (): I18nContextType => {
  try {
    return useContext(I18nContext);
  } catch {
    return {
      language: "system",
      effectiveLocale: "en",
      setLanguage: () => {},
      t: en,
    };
  }
};
