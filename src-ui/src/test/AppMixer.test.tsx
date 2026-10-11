import { renderToStaticMarkup } from "react-dom/server";
import {
  AppMixerDrawer,
  ProcessAudioSession,
  volumeToDbString,
} from "../components/AppMixerDrawer";

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

export function runAppMixerTests(): TestResult[] {
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

  const sampleSessions: ProcessAudioSession[] = [
    {
      process_id: 1001,
      name: "spotify.exe",
      volume: 0.85,
      is_muted: false,
      is_active: true,
    },
    {
      process_id: 1002,
      name: "discord.exe",
      volume: 0.70,
      is_muted: false,
      is_active: true,
    },
    {
      process_id: 1003,
      name: "chrome.exe",
      volume: 1.00,
      is_muted: false,
      is_active: false,
    },
  ];

  test("1. volumeToDbString computes exact perceptual decibel values", () => {
    assert(volumeToDbString(1.0, false) === "0.0 dB", "1.0 must be 0.0 dB");
    assert(volumeToDbString(0.5, false) === "-12.0 dB", "0.5 must be -12.0 dB");
    assert(volumeToDbString(0.0, false) === "-inf dB", "0.0 must be -inf dB");
    assert(volumeToDbString(0.85, true) === "-inf dB", "Muted volume must be -inf dB");
    assert(volumeToDbString(1.35, false) === "+2.6 dB", "1.35 boost must be +2.6 dB");
  });

  test("2. AppMixerDrawer hidden state when isOpen is false", () => {
    const html = renderToStaticMarkup(
      <AppMixerDrawer
        isOpen={false}
        onClose={() => {}}
        sessions={sampleSessions}
        onVolumeChange={() => {}}
        onToggleMute={() => {}}
      />
    );
    assert(html.includes('aria-hidden="true"'), "Drawer must have aria-hidden='true' when closed");
    assert(html.includes("display:none"), "Drawer must have display:none style when closed");
    assert(html.includes("translate-y-full"), "Drawer must have translate-y-full when closed");
  });

  test("3. AppMixerDrawer open state renders header and session count", () => {
    const html = renderToStaticMarkup(
      <AppMixerDrawer
        isOpen={true}
        onClose={() => {}}
        sessions={sampleSessions}
        onVolumeChange={() => {}}
        onToggleMute={() => {}}
      />
    );
    assert(html.includes('role="dialog"'), "Drawer must have role='dialog'");
    assert(html.includes("APPLICATION STREAM MIXER"), "Must display header title");
    assert(html.includes("3 Sessions"), "Must display 3 Sessions badge");
  });

  test("4. Renders each application row with process metadata and PID", () => {
    const html = renderToStaticMarkup(
      <AppMixerDrawer
        isOpen={true}
        onClose={() => {}}
        sessions={sampleSessions}
        onVolumeChange={() => {}}
        onToggleMute={() => {}}
      />
    );
    assert(html.includes("data-testid=\"app-mixer-row-1001\""), "Must render row for PID 1001");
    assert(html.includes("data-testid=\"app-mixer-row-1002\""), "Must render row for PID 1002");
    assert(html.includes("data-testid=\"app-mixer-row-1003\""), "Must render row for PID 1003");
    assert(html.includes("Spotify"), "Must render Spotify friendly label");
    assert(html.includes("Discord"), "Must render Discord friendly label");
    assert(html.includes("PID 1001"), "Must display PID 1001 badge");
  });

  test("5. Renders active playback indicator dot for playing sessions", () => {
    const html = renderToStaticMarkup(
      <AppMixerDrawer
        isOpen={true}
        onClose={() => {}}
        sessions={sampleSessions}
        onVolumeChange={() => {}}
        onToggleMute={() => {}}
      />
    );
    assert(
      html.includes("title=\"Actively outputting audio\""),
      "Must have active audio tooltip for active session"
    );
    assert(
      html.includes("bg-meter-green"),
      "Must render emerald indicator dot for active session"
    );
  });

  test("6. Independent volume faders configure 0% to 150% range and badges", () => {
    const html = renderToStaticMarkup(
      <AppMixerDrawer
        isOpen={true}
        onClose={() => {}}
        sessions={sampleSessions}
        onVolumeChange={() => {}}
        onToggleMute={() => {}}
      />
    );
    assert(html.includes('min="0"'), "Slider must have min 0");
    assert(html.includes('max="1.5"'), "Slider must have max 1.5 (150%)");
    assert(html.includes("85%"), "Must display 85% for Spotify");
    assert(html.includes("70%"), "Must display 70% for Discord");
  });

  test("7. Muted application session dims slider and updates mute icon", () => {
    const mutedSessions: ProcessAudioSession[] = [
      {
        process_id: 1001,
        name: "spotify.exe",
        volume: 0.85,
        is_muted: true,
        is_active: true,
      },
    ];

    const html = renderToStaticMarkup(
      <AppMixerDrawer
        isOpen={true}
        onClose={() => {}}
        sessions={mutedSessions}
        onVolumeChange={() => {}}
        onToggleMute={() => {}}
      />
    );
    assert(html.includes('disabled=""'), "Slider must be disabled when muted");
    assert(html.includes("opacity-40"), "Slider must have opacity-40 when muted");
    assert(html.includes('aria-label="Unmute Spotify"'), "Button must reflect Unmute state");
  });

  test("8. Empty session list renders friendly guidance message", () => {
    const html = renderToStaticMarkup(
      <AppMixerDrawer
        isOpen={true}
        onClose={() => {}}
        sessions={[]}
        onVolumeChange={() => {}}
        onToggleMute={() => {}}
      />
    );
    assert(
      html.includes("No active audio streams detected"),
      "Must show empty state message"
    );
  });

  test("9. Footer renders keyboard shortcut and isolation notice", () => {
    const html = renderToStaticMarkup(
      <AppMixerDrawer
        isOpen={true}
        onClose={() => {}}
        sessions={sampleSessions}
        onVolumeChange={() => {}}
        onToggleMute={() => {}}
      />
    );
    assert(html.includes("Ctrl+Shift+M"), "Footer must state Ctrl+Shift+M shortcut");
    assert(
      html.includes("Core Audio MTA Isolated"),
      "Footer must show isolation guarantee"
    );
  });

  return results;
}
