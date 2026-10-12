import { renderToStaticMarkup } from "react-dom/server";
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

export function runTitlebarVolumeSyncTests(): TestResult[] {
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

  const dummyDevices = [
    { id: "default", name: "Speakers (Realtek)", is_default: true },
  ];

  test("1. Renders Titlebar with 45% volume (0.45) reflecting OS volume adjustment", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <Titlebar
          masterPower={true}
          onTogglePower={() => {}}
          volume={0.45}
          onVolumeChange={() => {}}
          isMuted={false}
          onToggleMute={() => {}}
          devices={dummyDevices}
          activeDeviceId="default"
          onSelectDevice={() => {}}
        />
      </I18nProvider>
    );

    assert(html.includes('aria-valuenow="0.45"'), "Slider must reflect 0.45 scalar volume");
    assert(html.includes('aria-valuemax="1.5"'), "Slider max range must be 1.5 (150% boost)");
    assert(html.includes('aria-valuemin="0"'), "Slider min range must be 0");
    assert(html.includes("OPENBOOM"), "Titlebar must display brand logo");
  });

  test("2. Renders 100% volume at unity gain (0.0 dB)", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <Titlebar
          masterPower={true}
          onTogglePower={() => {}}
          volume={1.0}
          onVolumeChange={() => {}}
          isMuted={false}
          onToggleMute={() => {}}
          devices={dummyDevices}
          activeDeviceId="default"
          onSelectDevice={() => {}}
        />
      </I18nProvider>
    );

    assert(html.includes('aria-valuenow="1"'), "Slider must reflect 1.0 scalar volume");
    assert(html.includes("0.0 dB"), "1.0 volume must display 0.0 dB");
  });

  test("3. Renders 135% boost volume (+2.6 dB)", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <Titlebar
          masterPower={true}
          onTogglePower={() => {}}
          volume={1.35}
          onVolumeChange={() => {}}
          isMuted={false}
          onToggleMute={() => {}}
          devices={dummyDevices}
          activeDeviceId="default"
          onSelectDevice={() => {}}
        />
      </I18nProvider>
    );

    assert(html.includes('aria-valuenow="1.35"'), "Slider must reflect 1.35 boost volume");
    assert(html.includes("+2.6 dB"), "1.35 volume must display +2.6 dB boost");
  });

  test("4. Muted state displays -inf dB and 0 scalar value", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <Titlebar
          masterPower={true}
          onTogglePower={() => {}}
          volume={0.8}
          onVolumeChange={() => {}}
          isMuted={true}
          onToggleMute={() => {}}
          devices={dummyDevices}
          activeDeviceId="default"
          onSelectDevice={() => {}}
        />
      </I18nProvider>
    );

    assert(html.includes('aria-valuenow="0"'), "Muted slider must reflect 0");
    assert(html.includes("-inf dB"), "Muted slider must display -inf dB");
  });

  return results;
}
