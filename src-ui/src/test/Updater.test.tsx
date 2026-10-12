import { renderToStaticMarkup } from "react-dom/server";
import { SettingsModal, UpdateInfo } from "../components/SettingsModal";
import { Titlebar } from "../components/Titlebar";

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

export function runUpdaterTests(): TestResult[] {
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

  test("Titlebar hides update badge when hasUpdate is false", () => {
    const html = renderToStaticMarkup(
      <Titlebar
        masterPower={true}
        onTogglePower={() => {}}
        volume={0.8}
        onVolumeChange={() => {}}
        isMuted={false}
        onToggleMute={() => {}}
        devices={[]}
        activeDeviceId="default"
        onSelectDevice={() => {}}
        hasUpdate={false}
      />
    );
    assert(!html.includes("Update Available"), "Should not show update button when hasUpdate is false");
  });

  test("Titlebar displays distinct neon update badge when hasUpdate is true", () => {
    const html = renderToStaticMarkup(
      <Titlebar
        masterPower={true}
        onTogglePower={() => {}}
        volume={0.8}
        onVolumeChange={() => {}}
        isMuted={false}
        onToggleMute={() => {}}
        devices={[]}
        activeDeviceId="default"
        onSelectDevice={() => {}}
        hasUpdate={true}
      />
    );
    assert(html.includes("Update Available") || html.includes("Update"), "Should render update button badge");
    assert(html.includes("text-amber-400"), "Should render with distinct amber/neon color");
    assert(html.includes("animate-pulse"), "Should render with subtle attention pulse");
  });

  test("SettingsModal renders Updates tab button and badge indicator", () => {
    const updateInfo: UpdateInfo = {
      isAvailable: true,
      currentVersion: "0.3.0",
      latestVersion: "0.4.0",
      releaseNotes: "• Feature 1\n• Feature 2",
      publishedAt: "2026-10-12T12:00:00Z",
      downloadUrl: "https://github.com/JVGS1111/open-boom/releases/download/v0.4.0/OpenBoom_setup.exe",
    };

    const html = renderToStaticMarkup(
      <SettingsModal
        isOpen={true}
        onClose={() => {}}
        launchOnStartup={false}
        onToggleStartup={() => {}}
        initialTab="updates"
        updateInfo={updateInfo}
      />
    );

    assert(html.includes("Application Updates"), "Should render Updates title");
    assert(html.includes("0.3.0"), "Should display current version");
    assert(html.includes("0.4.0"), "Should display latest version");
    assert(html.includes("New version available!"), "Should render new version available banner");
    assert(html.includes("Download &amp; Install Update"), "Should display download action button");
    assert(html.includes("Check for updates on launch"), "Should display startup check toggle");
    assert(html.includes("Automatically install updates"), "Should display auto-install toggle");
  });

  test("SettingsModal renders Up to Date status when isAvailable is false", () => {
    const updateInfo: UpdateInfo = {
      isAvailable: false,
      currentVersion: "0.3.0",
      latestVersion: "0.3.0",
    };

    const html = renderToStaticMarkup(
      <SettingsModal
        isOpen={true}
        onClose={() => {}}
        launchOnStartup={false}
        onToggleStartup={() => {}}
        initialTab="updates"
        updateInfo={updateInfo}
      />
    );

    assert(html.includes("OpenBoom is up to date"), "Should display up to date message");
    assert(html.includes("Check for Updates"), "Should display manual check button");
  });

  test("SettingsModal renders Restart Application button when update is downloaded", () => {
    const html = renderToStaticMarkup(
      <SettingsModal
        isOpen={true}
        onClose={() => {}}
        launchOnStartup={false}
        onToggleStartup={() => {}}
        initialTab="updates"
        isUpdateReadyToRestart={true}
      />
    );

    assert(html.includes("Restart Application"), "Should display restart button when ready");
    assert(html.includes("bg-emerald-500"), "Should display prominent restart action styling");
  });

  return results;
}
