import { renderToStaticMarkup } from "react-dom/server";
import { SettingsModal } from "../components/SettingsModal";

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

export function runSettingsModalTests(): TestResult[] {
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

  test("SettingsModal returns null when isOpen is false", () => {
    const html = renderToStaticMarkup(
      <SettingsModal
        isOpen={false}
        onClose={() => {}}
        launchOnStartup={false}
        onToggleStartup={() => {}}
      />
    );
    assert(html === "", "Expected empty string when modal is closed");
  });

  test("SettingsModal renders core preferences and startup switch when open", () => {
    const html = renderToStaticMarkup(
      <SettingsModal
        isOpen={true}
        onClose={() => {}}
        launchOnStartup={true}
        onToggleStartup={() => {}}
      />
    );
    assert(html.includes("Preferences &amp; System"), "Should display modal header");
    assert(html.includes("Launch on Windows Startup"), "Should display Windows startup section");
    assert(html.includes("Atomic Persistence Contract"), "Should display atomic storage section");
  });

  test("Audio Buffer Stability Guard renders active protection badge when enabled", () => {
    const html = renderToStaticMarkup(
      <SettingsModal
        isOpen={true}
        onClose={() => {}}
        launchOnStartup={false}
        onToggleStartup={() => {}}
        bufferGuardEnabled={true}
        onToggleBufferGuard={() => {}}
      />
    );
    assert(
      html.includes("Audio Buffer Stability Guard"),
      "Should render Audio Buffer Stability Guard section"
    );
    assert(
      html.includes("aria-checked=\"true\"") && html.includes("Audio Buffer Stability Guard"),
      "Toggle switch should be checked"
    );
    assert(
      html.includes("Proteção Ativa") || html.includes("Active Protection"),
      "Should render active protection badge"
    );
    assert(
      !html.includes("Aviso de Risco") && !html.includes("Acoustic Risk"),
      "Should not render warning banner when guard is enabled"
    );
  });

  test("Audio Buffer Stability Guard displays prominent red warning banner when disabled", () => {
    const html = renderToStaticMarkup(
      <SettingsModal
        isOpen={true}
        onClose={() => {}}
        launchOnStartup={false}
        onToggleStartup={() => {}}
        bufferGuardEnabled={false}
        onToggleBufferGuard={() => {}}
      />
    );
    assert(
      html.includes("aria-checked=\"false\"") && html.includes("Audio Buffer Stability Guard"),
      "Toggle switch should be unchecked"
    );
    assert(
      html.includes("Aviso de Risco") || html.includes("Acoustic Risk"),
      "Must render prominent warning header when disabled"
    );
    assert(
      html.includes("10ms"),
      "Must explain the aggressive 10ms low latency mode risks"
    );
    assert(
      !html.includes("Proteção Ativa") && !html.includes("Active Protection"),
      "Must not render active badge when disabled"
    );
  });

  test("Audio Capture Bridge Mode selector renders options and active virtual channel status", () => {
    const html = renderToStaticMarkup(
      <SettingsModal
        isOpen={true}
        onClose={() => {}}
        launchOnStartup={false}
        onToggleStartup={() => {}}
        audioBridgeMode="virtual"
        usingVirtualBridge={true}
      />
    );
    assert(
      html.includes("value=\"auto\"") &&
        html.includes("value=\"virtual\"") &&
        html.includes("value=\"loopback\""),
      "Should render all three audio bridge mode select options (auto, virtual, loopback)"
    );
    assert(
      html.includes("Mode A") || html.includes("Modo A"),
      "Should display active Mode A virtual sink status"
    );
  });

  test("Audio Capture Bridge Mode selector displays amber warning banner when loopback is selected", () => {
    const html = renderToStaticMarkup(
      <SettingsModal
        isOpen={true}
        onClose={() => {}}
        launchOnStartup={false}
        onToggleStartup={() => {}}
        audioBridgeMode="loopback"
        usingVirtualBridge={false}
      />
    );
    assert(
      html.includes("Mode B (Driverless Loopback)") ||
        html.includes("Modo B (Emulação Loopback Sem Driver)"),
      "Should display Mode B driverless loopback warning banner when loopback mode is selected"
    );
  });

  return results;
}
