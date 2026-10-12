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

  test("Audio Capture Bridge Mode selector renders options, selected value, and active Mode A virtual status badge", () => {
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
      /<option[^>]*value="virtual"[^>]*selected=""/.test(html),
      "Should mark the virtual option as selected when audioBridgeMode is virtual"
    );
    assert(
      html.includes("Active Channel: Mode A") || html.includes("Canal Ativo: Modo A"),
      "Should display the exact Active Channel: Mode A virtual sink status badge"
    );
    assert(
      !html.includes("data-testid=\"audio-bridge-loopback-warning\""),
      "Should not render loopback warning banner when virtual bridge is active in Mode A"
    );
  });

  test("Audio Capture Bridge Mode selector displays both Mode B badge and amber warning banner when loopback is selected or active", () => {
    const htmlLoopback = renderToStaticMarkup(
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
      /<option[^>]*value="loopback"[^>]*selected=""/.test(htmlLoopback),
      "Should mark the loopback option as selected when audioBridgeMode is loopback"
    );
    assert(
      htmlLoopback.includes("Active Channel: Mode B") ||
        htmlLoopback.includes("Canal Ativo: Modo B"),
      "Should display Active Channel: Mode B badge when loopback is selected"
    );
    assert(
      htmlLoopback.includes("Mode B (Driverless Loopback)") ||
        htmlLoopback.includes("Modo B (Emulação Loopback Sem Driver)"),
      "Should display Mode B driverless loopback warning banner when loopback mode is selected"
    );

    const htmlAutoFallback = renderToStaticMarkup(
      <SettingsModal
        isOpen={true}
        onClose={() => {}}
        launchOnStartup={false}
        onToggleStartup={() => {}}
        audioBridgeMode="auto"
        usingVirtualBridge={false}
        isSwitchingBridge={true}
      />
    );
    assert(
      htmlAutoFallback.includes("Active Channel: Mode B") ||
        htmlAutoFallback.includes("Canal Ativo: Modo B"),
      "Should display Active Channel: Mode B badge when auto mode falls back to loopback"
    );
    assert(
      htmlAutoFallback.includes("data-testid=\"audio-bridge-loopback-warning\""),
      "Should display warning banner when auto mode falls back to Mode B loopback"
    );
    assert(
      htmlAutoFallback.includes("disabled=\"\""),
      "Should disable select control while bridge mode switch/restart is in flight"
    );
  });

  return results;
}
