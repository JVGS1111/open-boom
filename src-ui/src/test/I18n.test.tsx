import { renderToStaticMarkup } from "react-dom/server";
import {
  resolveEffectiveLocale,
  I18nProvider,
  useI18n,
  TRANSLATIONS,
  LANGUAGE_OPTIONS,
} from "../i18n";
import { LanguageMenu } from "../components/LanguageMenu";

export interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
}

function assert(condition: boolean, message: string) {
  if (!condition) {
    throw new Error(message);
  }
}

function verifyDictionaryCompleteness(
  source: Record<string, any>,
  target: Record<string, any>,
  path = ""
) {
  for (const key of Object.keys(source)) {
    const currentPath = path ? `${path}.${key}` : key;
    assert(
      key in target,
      `Missing translation key: "${currentPath}" in target dictionary`
    );
    if (typeof source[key] === "object" && source[key] !== null) {
      assert(
        typeof target[key] === "object" && target[key] !== null,
        `Expected object at "${currentPath}"`
      );
      verifyDictionaryCompleteness(source[key], target[key], currentPath);
    } else {
      assert(
        typeof target[key] === "string" && target[key].length > 0,
        `Empty or non-string translation at "${currentPath}"`
      );
    }
  }
}

export function runI18nTests(): TestResult[] {
  const results: TestResult[] = [];

  function test(name: string, fn: () => void) {
    try {
      fn();
      results.push({ name, passed: true });
    } catch (err) {
      results.push({
        name,
        passed: false,
        error: err instanceof Error ? err.message : String(err),
      });
    }
  }

  test("1. resolveEffectiveLocale maps options accurately", () => {
    assert(resolveEffectiveLocale("pt-BR") === "pt-BR", "Must resolve pt-BR");
    assert(resolveEffectiveLocale("es") === "es", "Must resolve es");
    assert(resolveEffectiveLocale("en") === "en", "Must resolve en");
    // "system" resolves to a valid SupportedLocale
    const sysLocale = resolveEffectiveLocale("system");
    assert(
      sysLocale === "pt-BR" || sysLocale === "en" || sysLocale === "es",
      `system resolved to invalid locale: ${sysLocale}`
    );
  });

  test("2. Portuguese dictionary (pt-BR) has full parity with English dictionary", () => {
    verifyDictionaryCompleteness(TRANSLATIONS.en, TRANSLATIONS["pt-BR"]);
  });

  test("3. Spanish dictionary (es) has full parity with English dictionary", () => {
    verifyDictionaryCompleteness(TRANSLATIONS.en, TRANSLATIONS.es);
  });

  test("4. Language options contains SYS, PT, EN, and ES badges", () => {
    const badges = LANGUAGE_OPTIONS.map((opt) => opt.badge);
    assert(badges.includes("SYS"), "Must include SYS badge");
    assert(badges.includes("PT"), "Must include PT badge");
    assert(badges.includes("EN"), "Must include EN badge");
    assert(badges.includes("ES"), "Must include ES badge");
  });

  test("5. I18nProvider provides pt-BR dictionary when configured", () => {
    const TestConsumer = () => {
      const { t, effectiveLocale } = useI18n();
      return (
        <div>
          <span data-testid="locale">{effectiveLocale}</span>
          <span data-testid="msg">{t.settings.modalTitle}</span>
        </div>
      );
    };

    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="pt-BR">
        <TestConsumer />
      </I18nProvider>
    );

    assert(html.includes("pt-BR"), "Must have effective locale pt-BR");
    assert(
      html.includes("Preferências &amp; Sistema") || html.includes("Preferências"),
      "Must render Portuguese settings title"
    );
  });

  test("6. I18nProvider provides Spanish dictionary when configured", () => {
    const TestConsumer = () => {
      const { t, effectiveLocale } = useI18n();
      return (
        <div>
          <span data-testid="locale">{effectiveLocale}</span>
          <span data-testid="msg">{t.settings.modalTitle}</span>
        </div>
      );
    };

    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="es">
        <TestConsumer />
      </I18nProvider>
    );

    assert(html.includes("es"), "Must have effective locale es");
    assert(
      html.includes("Preferencias &amp; Sistema") || html.includes("Preferencias"),
      "Must render Spanish settings title"
    );
  });

  test("7. LanguageMenu renders trigger button and active language badge", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="pt-BR">
        <LanguageMenu />
      </I18nProvider>
    );

    assert(html.includes("PT"), "Should display PT badge for active pt-BR language");
    assert(html.includes("<button"), "Should render menu trigger button");
  });

  return results;
}
