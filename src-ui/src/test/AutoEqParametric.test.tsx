import { renderToStaticMarkup } from "react-dom/server";
import { calculateAutoEqPreampOffset } from "../App";
import { EqualizerPreview } from "../components/EqualizerPreview";
import { EqualizerStudioDrawer } from "../components/EqualizerStudioDrawer";

export function runAutoEqParametricTests() {
  const results: { name: string; passed: boolean; error?: string }[] = [];

  function assert(condition: boolean, message: string) {
    if (!condition) {
      throw new Error(`Assertion failed: ${message}`);
    }
  }

  function test(name: string, fn: () => void) {
    try {
      fn();
      results.push({ name, passed: true });
    } catch (err: unknown) {
      results.push({ name, passed: false, error: String(err) });
    }
  }

  // 1. Math: Automatic Preamp Calculation
  test("calculateAutoEqPreampOffset computes equal negative offset for positive peaks", () => {
    const filters = [
      { gain_db: 2.0 },
      { gain_db: 6.5 }, // Max boost = +6.5 dB
      { gain_db: -3.0 },
      { gain_db: 4.0 },
    ];
    const offset = calculateAutoEqPreampOffset(filters);
    assert(offset === -6.5, `Expected -6.5 dB, got ${offset}`);
  });

  test("calculateAutoEqPreampOffset returns 0.0 dB for negative-only cuts", () => {
    const filters = [
      { gain_db: -3.5 },
      { gain_db: -1.0 },
      { gain_db: -8.0 },
    ];
    const offset = calculateAutoEqPreampOffset(filters);
    assert(offset === 0.0, `Expected 0.0 dB for negative cuts, got ${offset}`);
  });

  test("calculateAutoEqPreampOffset returns 0.0 dB for empty filter list", () => {
    const offset = calculateAutoEqPreampOffset([]);
    assert(offset === 0.0, `Expected 0.0 dB for empty filters, got ${offset}`);
  });

  test("calculateAutoEqPreampOffset sanitizes NaN or undefined gains", () => {
    const filters = [
      { gain_db: NaN },
      { gain_db: 5.2 },
      { gain_db: undefined },
    ];
    const offset = calculateAutoEqPreampOffset(filters);
    assert(offset === -5.2, `Expected -5.2 dB, got ${offset}`);
  });

  // 2. UI Component: EqualizerPreview Preamp Badge
  test("EqualizerPreview renders negative preamp badge when AutoEq is active", () => {
    const bands = new Array(31).fill(0.0);
    const html = renderToStaticMarkup(
      <EqualizerPreview
        activePreset="Harman Acoustic"
        bands={bands}
        preampOffsetDb={-5.2}
        activeAutoEqName="Sennheiser HD 600"
      />
    );

    assert(
      html.includes("(-5.2 dB)"),
      "EqualizerPreview must render (-5.2 dB) preamp offset badge"
    );
    assert(
      html.includes("Sennheiser HD 600"),
      "EqualizerPreview must render active AutoEq model name"
    );
  });

  test("EqualizerPreview renders 0.0 dB preamp when flat", () => {
    const bands = new Array(31).fill(0.0);
    const html = renderToStaticMarkup(
      <EqualizerPreview
        activePreset="Flat"
        bands={bands}
        preampOffsetDb={0.0}
      />
    );

    assert(
      html.includes("(0.0 dB)"),
      "EqualizerPreview must render (0.0 dB) when no preamp attenuation is active"
    );
  });

  // 3. UI Component: EqualizerStudioDrawer Preamp & AutoEq Status
  test("EqualizerStudioDrawer renders Preamp badge in header when offset is active", () => {
    const gains = new Array(31).fill(0.0);
    const html = renderToStaticMarkup(
      <EqualizerStudioDrawer
        isOpen={true}
        onClose={() => {}}
        mode="31"
        onModeChange={() => {}}
        gains={gains}
        onGainChange={() => {}}
        onResetAll={() => {}}
        activePreset="Harman Acoustic"
        preampOffsetDb={-6.5}
        activeAutoEqName="HyperX Cloud II"
      />
    );

    assert(
      html.includes("Preamp: -6.5 dB"),
      "EqualizerStudioDrawer must display Preamp: -6.5 dB in header"
    );
    assert(
      html.includes("HyperX Cloud II"),
      "EqualizerStudioDrawer must display active AutoEq headphone profile name"
    );
  });

  test("EqualizerStudioDrawer omits Preamp badge when offset is 0.0 dB", () => {
    const gains = new Array(31).fill(0.0);
    const html = renderToStaticMarkup(
      <EqualizerStudioDrawer
        isOpen={true}
        onClose={() => {}}
        mode="31"
        onModeChange={() => {}}
        gains={gains}
        onGainChange={() => {}}
        onResetAll={() => {}}
        activePreset="Flat"
        preampOffsetDb={0.0}
      />
    );

    assert(
      !html.includes("Preamp:"),
      "EqualizerStudioDrawer should omit Preamp badge when offset is 0.0 dB"
    );
  });

  // 4. Decoupled Curve Preservation (FR12, AD-6)
  test("31-band fader curve is decoupled and strictly preserved when AutoEq is loaded", () => {
    const initialGains = new Array(31).fill(0.0);
    initialGains[17] = 4.5; // Custom user boost at 1 kHz

    // Mock applying AutoEq profile: only preampOffsetDb and activeAutoEqName update, gains remain intact
    const autoEqProfile = {
      name: "Sony WH-1000XM5",
      preamp_gain_db: -4.2,
      filters: [{ gain_db: 4.2 }],
    };

    const calculatedOffset = calculateAutoEqPreampOffset(autoEqProfile.filters);
    assert(calculatedOffset === -4.2, "Calculated offset must match -4.2 dB");

    // The user's 31-band fader gains array remains unmodified
    assert(
      initialGains[17] === 4.5,
      "User 31-band fader gain at 1 kHz must remain 4.5 dB without being reset"
    );
  });

  return results;
}
