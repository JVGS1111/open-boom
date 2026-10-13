import { renderToStaticMarkup } from "react-dom/server";
import { DriverOnboardingModal } from "../components/DriverOnboardingModal";
import { Titlebar } from "../components/Titlebar";
import { I18nProvider } from "../i18n";

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

export function runDriverOnboardingTests(): TestResult[] {
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

  test("1. DriverOnboardingModal returns null/empty when isOpen is false", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <DriverOnboardingModal isOpen={false} onClose={() => {}} />
      </I18nProvider>
    );
    assert(html === "", "Expected empty string when modal is closed");
  });

  test("2. DriverOnboardingModal renders title, features, and 1-click button when open", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <DriverOnboardingModal isOpen={true} onClose={() => {}} />
      </I18nProvider>
    );
    assert(html.includes("7.1 Surround Audio Driver Required"), "Should render modal title");
    assert(html.includes("Discrete 7.1 Surround (8 Channels)"), "Should render 7.1 feature badge");
    assert(html.includes("Zero Acoustic Duplication or Echo"), "Should render no echo feature badge");
    assert(html.includes("Install 7.1 Surround Device (1-Click)"), "Should render install button");
    assert(html.includes("Manual driver download"), "Should render manual download fallback");
  });

  test("3. DriverOnboardingModal renders error alert and retry button when installError is present", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <DriverOnboardingModal
          isOpen={true}
          onClose={() => {}}
          installError="Administrator permission (UAC) was cancelled."
          onRetry={() => {}}
        />
      </I18nProvider>
    );
    assert(html.includes("Driver Installation Failed"), "Should render error alert header");
    assert(html.includes("Administrator permission (UAC) was cancelled."), "Should render error message");
    assert(html.includes("Try Again"), "Should render retry button");
  });

  test("4. DriverOnboardingModal renders installing spinner state", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <DriverOnboardingModal
          isOpen={true}
          onClose={() => {}}
          isInstalling={true}
        />
      </I18nProvider>
    );
    assert(html.includes("Installing Driver"), "Should render installing status text");
    assert(html.includes("animate-spin"), "Should render spinning icon");
  });

  test("5. Titlebar applies disabled gating styling when disabled is true", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <Titlebar
          masterPower={true}
          onTogglePower={() => {}}
          volume={0.8}
          onVolumeChange={() => {}}
          isMuted={false}
          onToggleMute={() => {}}
          devices={[{ id: "dev-1", name: "Speakers", is_default: true }]}
          activeDeviceId="dev-1"
          onSelectDevice={() => {}}
          disabled={true}
        />
      </I18nProvider>
    );
    assert(
      html.includes("opacity-40 pointer-events-none select-none cursor-not-allowed"),
      "Controls container should receive disabled gating classes"
    );
  });

  test("6. DriverOnboardingModal renders localized strings in Portuguese (pt-BR)", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="pt-BR">
        <DriverOnboardingModal isOpen={true} onClose={() => {}} />
      </I18nProvider>
    );
    assert(
      html.includes("Driver de Áudio Surround 7.1 Necessário"),
      "Should render Portuguese modal title"
    );
    assert(
      html.includes("Instalar Dispositivo Surround 7.1 (1 Clique)"),
      "Should render Portuguese install button"
    );
  });

  test("7. DriverOnboardingModal renders Spanish error state and external fallback link", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="es">
        <DriverOnboardingModal
          isOpen={true}
          onClose={() => {}}
          installError="El permiso de administrador (UAC) fue cancelado o denegado."
          onRetry={() => {}}
        />
      </I18nProvider>
    );
    assert(
      html.includes("Fallo en la Instalación del Controlador"),
      "Should render Spanish error alert title"
    );
    assert(
      html.includes("El permiso de administrador (UAC) fue cancelado o denegado."),
      "Should render Spanish error message"
    );
    assert(
      html.includes("Intentar de Nuevo"),
      "Should render Spanish retry button"
    );
    assert(
      html.includes("Descarga manual del controlador (Sitio oficial)"),
      "Should render Spanish manual download link"
    );
  });

  return results;
}
