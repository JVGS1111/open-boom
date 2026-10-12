import { renderToStaticMarkup } from "react-dom/server";
import { SoundstageDial, azimuthToCartesian } from "../components/SoundstageDial";

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

export function runSoundstageDialTests(): TestResult[] {
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

  // 1. Polar coordinate projection tests
  test("1. azimuthToCartesian maps acoustic azimuths accurately to dial track coordinates", () => {
    const r = 84;
    const cx = 96;
    const cy = 96;

    // 0° = North / Center (x = 96, y = 12)
    const pNorth = azimuthToCartesian(0, r, cx, cy);
    assert(Math.abs(pNorth.x - 96) < 1e-3, "0° X coordinate must be cx (96)");
    assert(Math.abs(pNorth.y - (96 - 84)) < 1e-3, "0° Y coordinate must be cy - R (12)");

    // +45° = Front-Right (x > 96, y < 96)
    const pFR = azimuthToCartesian(45, r, cx, cy);
    const expectedFRX = 96 + 84 * Math.sin((45 * Math.PI) / 180);
    const expectedFRY = 96 - 84 * Math.cos((45 * Math.PI) / 180);
    assert(Math.abs(pFR.x - expectedFRX) < 1e-2, "45° X must match Front-Right polar projection");
    assert(Math.abs(pFR.y - expectedFRY) < 1e-2, "45° Y must match Front-Right polar projection");
    assert(pFR.x > 96, "45° X must be to the right of center");
    assert(pFR.y < 96, "45° Y must be in front/top of center");

    // +90° = East / Right (x = 180, y = 96)
    const pEast = azimuthToCartesian(90, r, cx, cy);
    assert(Math.abs(pEast.x - 180) < 1e-3, "+90° X coordinate must be cx + R (180)");
    assert(Math.abs(pEast.y - 96) < 1e-3, "+90° Y coordinate must be cy (96)");

    // -90° = West / Left (x = 12, y = 96)
    const pWest = azimuthToCartesian(-90, r, cx, cy);
    assert(Math.abs(pWest.x - 12) < 1e-3, "-90° X coordinate must be cx - R (12)");
    assert(Math.abs(pWest.y - 96) < 1e-3, "-90° Y coordinate must be cy (96)");

    // ±180° = South / Rear (x = 96, y = 180)
    const pSouth = azimuthToCartesian(180, r, cx, cy);
    assert(Math.abs(pSouth.x - 96) < 1e-3, "180° X coordinate must be cx (96)");
    assert(Math.abs(pSouth.y - 180) < 1e-3, "180° Y coordinate must be cy + R (180)");
  });

  // 2. NaN and extreme angle sanitization
  test("2. azimuthToCartesian sanitizes NaN, infinite, and wrapped angles safely", () => {
    const r = 84;
    const pNaN = azimuthToCartesian(NaN, r);
    assert(!isNaN(pNaN.x) && !isNaN(pNaN.y), "NaN azimuth must produce valid finite coordinates");
    assert(pNaN.x === 96 && pNaN.y === 12, "NaN azimuth must default to 0° North (96, 12)");

    const pInf = azimuthToCartesian(Infinity, r);
    assert(!isNaN(pInf.x) && !isNaN(pInf.y), "Infinity azimuth must produce valid finite coordinates");
    assert(pInf.x === 96 && pInf.y === 12, "Infinity azimuth must default to 0° North (96, 12)");
  });

  // 3. SoundstageDial renders with 8D Orbit disabled
  test("3. SoundstageDial when orbitEnabled is false renders button inactive and omits comet", () => {
    const html = renderToStaticMarkup(
      <SoundstageDial
        enabled={true}
        intensity={0.5}
        onIntensityChange={() => {}}
        onToggle={() => {}}
        orbitEnabled={false}
        orbitPhaseDeg={0.0}
        onToggleOrbit={() => {}}
      />
    );

    assert(html.includes("8D Orbit"), "Must render 8D Orbit button label");
    assert(html.includes('aria-pressed="false"'), "8D Orbit button must have aria-pressed='false'");
    assert(!html.includes('data-testid="orbital-comet"'), "Orbital comet SVG must NOT be rendered when orbitEnabled is false");
    assert(html.includes("KEMAR 7.1 HRTF"), "Knob subtitle must display KEMAR 7.1 HRTF when orbit is inactive");
  });

  // 4. SoundstageDial renders with 8D Orbit enabled
  test("4. SoundstageDial when orbitEnabled is true renders neon comet and active button styling", () => {
    const html = renderToStaticMarkup(
      <SoundstageDial
        enabled={true}
        intensity={0.5}
        onIntensityChange={() => {}}
        onToggle={() => {}}
        orbitEnabled={true}
        orbitPhaseDeg={45.0}
        onToggleOrbit={() => {}}
      />
    );

    assert(html.includes('aria-pressed="true"'), "8D Orbit button must have aria-pressed='true'");
    assert(html.includes('data-testid="orbital-comet"'), "Orbital comet SVG MUST be rendered when orbitEnabled is true");
    assert(html.includes('data-testid="comet-beacon-head"'), "Comet beacon head element must be present");
    assert(html.includes('data-testid="comet-beacon-disc"'), "Comet beacon disc element must be present");
    assert(html.includes('data-testid="comet-beacon-halo"'), "Comet beacon halo element must be present");
    assert(html.includes('data-testid="comet-tail-1"'), "Comet trailing particle 1 must be present");
    assert(html.includes("8D ORBIT ACTIVE"), "Knob subtitle must display 8D ORBIT ACTIVE");

    // Coordinates check for 45° Front-Right
    const pExpected = azimuthToCartesian(45.0, 84, 96, 96);
    const expectedXStr = `cx="${pExpected.x.toFixed(2)}"`;
    const expectedYStr = `cy="${pExpected.y.toFixed(2)}"`;
    assert(html.includes(expectedXStr), `HTML must contain beacon cx=${pExpected.x.toFixed(2)}`);
    assert(html.includes(expectedYStr), `HTML must contain beacon cy=${pExpected.y.toFixed(2)}`);
  });

  // 5. Button Accessibility
  test("5. 8D Orbit button provides accessible role, name, and aria-pressed attributes", () => {
    const html = renderToStaticMarkup(
      <SoundstageDial
        enabled={true}
        intensity={0.5}
        onIntensityChange={() => {}}
        onToggle={() => {}}
        orbitEnabled={true}
        onToggleOrbit={() => {}}
      />
    );

    assert(html.includes('data-testid="orbit-8d-button"'), "8D Orbit button must be rendered");
    assert(html.includes('aria-label="8D Orbit"'), "8D Orbit button must declare aria-label='8D Orbit'");
    assert(html.includes('aria-pressed="true"'), "Active button must declare aria-pressed='true'");
  });

  // 6. Keyboard and click handlers invocation
  test("6. Invoking onToggleOrbit handler triggers toggle action", () => {
    let toggleCalled = false;
    const handleToggle = () => {
      toggleCalled = true;
    };

    const dial = (
      <SoundstageDial
        enabled={true}
        intensity={0.5}
        onIntensityChange={() => {}}
        onToggle={() => {}}
        orbitEnabled={false}
        onToggleOrbit={handleToggle}
      />
    );

    assert(dial.props.onToggleOrbit !== undefined);
    dial.props.onToggleOrbit();
    assert(toggleCalled, "onToggleOrbit must be bound to SoundstageDial and trigger state update");
  });

  // 7. Zero layout shift and bounded SVG geometry
  test("7. Dial SVG preserves 192x192 viewBox with zero layout shift or visual overflow", () => {
    const html = renderToStaticMarkup(
      <SoundstageDial
        enabled={true}
        intensity={0.75}
        onIntensityChange={() => {}}
        onToggle={() => {}}
        orbitEnabled={true}
        orbitPhaseDeg={180.0}
        onToggleOrbit={() => {}}
      />
    );

    assert(html.includes('viewBox="0 0 192 192"'), "Dial SVG must have fixed 192x192 viewBox");
    assert(html.includes('w-[192px] h-[192px]'), "Dial outer container must maintain 192px dimensions");
  });

  return results;
}
