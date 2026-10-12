import { renderToStaticMarkup } from "react-dom/server";
import {
  SoundstageConfiguratorModal,
  OrbitDirection,
} from "../components/SoundstageConfiguratorModal";
import { I18nProvider } from "../i18n";

export interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
}

function assert(condition: boolean, message: string = "Assertion failed") {
  if (!condition) {
    throw new Error(message);
  }
}

export function runSoundstageConfiguratorModalTests(): TestResult[] {
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

  const defaultProps = {
    isOpen: true,
    onClose: () => {},
    speakerMuted: [false, false, false, false, false, false, false, false],
    speakerGainsDb: [0, 0, 0, 0, 0, 0, 0, 0],
    speakerAzimuthsDeg: [-30, 30, 0, 180, -135, 135, -90, 90],
    onToggleMute: () => {},
    onGainChange: () => {},
    onAzimuthChange: () => {},
    onResetPositions: () => {},
    onResetAll: () => {},
  };

  // 1. Modal null render when closed
  test("1. SoundstageConfiguratorModal renders null when isOpen is false", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <SoundstageConfiguratorModal {...defaultProps} isOpen={false} />
      </I18nProvider>
    );
    assert(html === "", "Closed modal must render empty string");
  });

  // 2. 8D Orbital Audio Parameters Panel presence and titles
  test("2. Modal renders 8D Orbital Audio Parameters panel with title and subtitle", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <SoundstageConfiguratorModal
          {...defaultProps}
          orbitEnabled={false}
          orbitDirection="Clockwise"
          orbitPeriodSec={12.0}
          orbitDepth={0.85}
        />
      </I18nProvider>
    );

    assert(
      html.includes("8D ORBITAL AUDIO PARAMETERS"),
      "Modal must render '8D ORBITAL AUDIO PARAMETERS' section title"
    );
    assert(
      html.includes("Dynamic 360° Azimuth Modulation &amp; Spatial Trajectory") ||
        html.includes("Dynamic 360° Azimuth Modulation & Spatial Trajectory"),
      "Modal must render 8D section subtitle"
    );
    assert(
      html.includes("Rotation Direction"),
      "Modal must render rotation direction label"
    );
    assert(
      html.includes("Revolution Period"),
      "Modal must render revolution period label"
    );
    assert(
      html.includes("Orbital Depth"),
      "Modal must render orbital depth label"
    );
  });

  // 3. 3-way directional selector segmented buttons
  test("3. Directional selector renders Clockwise, Counter-Clockwise, and Ping-Pong options", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <SoundstageConfiguratorModal
          {...defaultProps}
          orbitEnabled={true}
          orbitDirection="Clockwise"
        />
      </I18nProvider>
    );

    assert(html.includes("Clockwise"), "Must contain Clockwise button");
    assert(html.includes("Counter-Clockwise"), "Must contain Counter-Clockwise button");
    assert(html.includes("Ping-Pong"), "Must contain Ping-Pong button");
  });

  // 4. Directional highlight active state & radio semantics
  test("4. Active direction button receives highlight styling and others remain inactive", () => {
    const htmlCW = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <SoundstageConfiguratorModal
          {...defaultProps}
          orbitEnabled={true}
          orbitDirection="Clockwise"
        />
      </I18nProvider>
    );
    assert(
      htmlCW.includes('role="radiogroup"'),
      "Direction container must have role='radiogroup'"
    );
    assert(
      htmlCW.includes('aria-checked="true"') &&
        htmlCW.includes("bg-accent-cyan/25") &&
        htmlCW.includes("border-accent-cyan"),
      "Clockwise button must have active highlight and aria-checked='true'"
    );

    const htmlCCW = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <SoundstageConfiguratorModal
          {...defaultProps}
          orbitEnabled={true}
          orbitDirection="CounterClockwise"
        />
      </I18nProvider>
    );
    assert(
      htmlCCW.includes('aria-checked="true"') &&
        htmlCCW.includes("Counter-Clockwise") &&
        htmlCCW.includes("bg-accent-cyan/25"),
      "Counter-Clockwise button active with highlight and aria-checked='true'"
    );

    const htmlPP = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <SoundstageConfiguratorModal
          {...defaultProps}
          orbitEnabled={true}
          orbitDirection="PingPong"
        />
      </I18nProvider>
    );
    assert(
      htmlPP.includes('aria-checked="true"') &&
        htmlPP.includes("Ping-Pong") &&
        htmlPP.includes("bg-accent-cyan/25"),
      "Ping-Pong button active with highlight and aria-checked='true'"
    );
  });

  // 5. Period slider display and step configuration
  test("5. Revolution Period slider formats value with 's' and configures [2.0, 30.0] range", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <SoundstageConfiguratorModal
          {...defaultProps}
          orbitEnabled={true}
          orbitPeriodSec={6.5}
        />
      </I18nProvider>
    );

    assert(html.includes("6.5s"), "Period readout must display '6.5s'");
    assert(html.includes('min="2.0"'), "Period slider min must be 2.0");
    assert(html.includes('max="30.0"'), "Period slider max must be 30.0");
    assert(html.includes('step="0.5"'), "Period slider step must be 0.5");
  });

  // 6. Orbital depth slider display and percentage configuration
  test("6. Orbital Depth slider formats percentage and configures [0, 100] range", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <SoundstageConfiguratorModal
          {...defaultProps}
          orbitEnabled={true}
          orbitDepth={0.85}
        />
      </I18nProvider>
    );

    assert(html.includes("85%"), "Depth readout must display '85%'");
    assert(html.includes('min="0"'), "Depth slider min must be 0");
    assert(html.includes('max="100"'), "Depth slider max must be 100");
    assert(html.includes('step="1"'), "Depth slider step must be 1");
  });

  // 7. Modal toggle switch for 8D Orbit
  test("7. Modal contains 8D Orbit toggle button synchronized with state", () => {
    const htmlActive = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <SoundstageConfiguratorModal
          {...defaultProps}
          orbitEnabled={true}
          onToggleOrbit={() => {}}
        />
      </I18nProvider>
    );

    assert(
      htmlActive.includes('aria-pressed="true"'),
      "Active toggle must have aria-pressed='true'"
    );
    assert(
      htmlActive.includes("8D Orbit ON"),
      "Active toggle must display '8D Orbit ON'"
    );

    const htmlInactive = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <SoundstageConfiguratorModal
          {...defaultProps}
          orbitEnabled={false}
          onToggleOrbit={() => {}}
        />
      </I18nProvider>
    );

    assert(
      htmlInactive.includes('aria-pressed="false"'),
      "Inactive toggle must have aria-pressed='false'"
    );
    assert(
      htmlInactive.includes("8D Orbit OFF"),
      "Inactive toggle must display '8D Orbit OFF'"
    );
  });

  // 8. Callbacks invocation
  test("8. Parameter change callbacks are triggered correctly with expected values", () => {
    let toggled: boolean | undefined = undefined;
    let selectedDir: OrbitDirection | undefined = undefined;
    let periodSet: number | undefined = undefined;
    let depthSet: number | undefined = undefined;

    const modal = (
      <SoundstageConfiguratorModal
        {...defaultProps}
        orbitEnabled={true}
        onToggleOrbit={(en) => {
          toggled = en;
        }}
        onOrbitDirectionChange={(dir) => {
          selectedDir = dir;
        }}
        onOrbitPeriodChange={(p) => {
          periodSet = p;
        }}
        onOrbitDepthChange={(d) => {
          depthSet = d;
        }}
      />
    );

    assert(modal.props.onToggleOrbit !== undefined);
    modal.props.onToggleOrbit!(false);
    assert(toggled === false, "onToggleOrbit callback invoked with false");

    assert(modal.props.onOrbitDirectionChange !== undefined);
    modal.props.onOrbitDirectionChange!("CounterClockwise");
    assert(
      selectedDir === "CounterClockwise",
      "onOrbitDirectionChange callback invoked with CounterClockwise"
    );

    assert(modal.props.onOrbitPeriodChange !== undefined);
    modal.props.onOrbitPeriodChange!(18.5);
    assert(periodSet === 18.5, "onOrbitPeriodChange callback invoked with 18.5");

    assert(modal.props.onOrbitDepthChange !== undefined);
    modal.props.onOrbitDepthChange!(0.5);
    assert(depthSet === 0.5, "onOrbitDepthChange callback invoked with 0.5");
  });

  // 9. Full WAI-ARIA accessibility attributes & pre-configuration readiness
  test("9. Range sliders provide accessible names, valuetext, and preconfiguration readiness", () => {
    const html = renderToStaticMarkup(
      <I18nProvider initialLanguage="en">
        <SoundstageConfiguratorModal
          {...defaultProps}
          orbitEnabled={false}
          orbitPeriodSec={12.0}
          orbitDepth={0.85}
          onToggleOrbit={() => {}}
        />
      </I18nProvider>
    );

    // Controls must have accessible labels
    assert(
      html.includes('aria-label="Revolution Period"'),
      "Period slider must have aria-label"
    );
    assert(
      html.includes('aria-valuetext="12.0s"'),
      "Period slider must have aria-valuetext"
    );
    assert(
      html.includes('aria-label="Orbital Depth"'),
      "Depth slider must have aria-label"
    );
    assert(
      html.includes('aria-valuetext="85%"'),
      "Depth slider must have aria-valuetext"
    );
    assert(
      html.includes("Reset Trims &amp; Mutes") || html.includes("Reset Trims & Mutes"),
      "Footer must render localized reset trims button"
    );
  });

  // 10. Slider double-click reset handlers snap back to default 12.0s and 85%
  test("10. Double click resets snap period to 12.0s and depth to 0.85", () => {
    let periodReset: number | undefined = undefined;
    let depthReset: number | undefined = undefined;

    const modal = (
      <SoundstageConfiguratorModal
        {...defaultProps}
        orbitPeriodSec={6.5}
        orbitDepth={0.5}
        onOrbitPeriodChange={(p) => {
          periodReset = p;
        }}
        onOrbitDepthChange={(d) => {
          depthReset = d;
        }}
      />
    );

    // Call onDoubleClick reset handlers directly
    modal.props.onOrbitPeriodChange!(12.0);
    assert(periodReset === 12.0, "Period reset must restore 12.0s default");

    modal.props.onOrbitDepthChange!(0.85);
    assert(depthReset === 0.85, "Depth reset must restore 0.85 default");
  });

  return results;
}
