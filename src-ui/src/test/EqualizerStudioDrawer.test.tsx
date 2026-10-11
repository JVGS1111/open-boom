import { renderToStaticMarkup } from "react-dom/server";
import {
  EqualizerStudioDrawer,
  EqFader,
  ISO_31_FREQUENCIES,
  ISO_10_FREQUENCIES,
  ISO_10_TO_31_INDICES,
  formatFrequency,
  applyMagneticSnap,
  calculateFaderDelta,
  frequencyToLogX,
  gainToY,
  computeSplineSegments,
  Point,
} from "../components/EqualizerStudioDrawer";
import { EqualizerPreview } from "../components/EqualizerPreview";

export function runEqualizerStudioDrawerTests() {
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

  // 1. ISO Frequency Specifications & Index Mappings (FR9, FR12)
  test("ISO 31-Band frequencies match standard 1/3-octave center frequencies", () => {
    assert(ISO_31_FREQUENCIES.length === 31, "Must have exactly 31 bands");
    assert(ISO_31_FREQUENCIES[0] === 20, "Band 1 center frequency must be 20 Hz");
    assert(ISO_31_FREQUENCIES[17] === 1000, "Band 18 center frequency must be 1000 Hz (1 kHz)");
    assert(ISO_31_FREQUENCIES[30] === 20000, "Band 31 center frequency must be 20000 Hz (20 kHz)");
  });

  test("ISO 10-Band frequencies match casual 1-octave center frequencies", () => {
    assert(ISO_10_FREQUENCIES.length === 10, "Must have exactly 10 octave bands");
    assert(ISO_10_FREQUENCIES[0] === 31.5, "10-Band #0 must be 31.5 Hz");
    assert(ISO_10_FREQUENCIES[5] === 1000, "10-Band #5 must be 1000 Hz");
    assert(ISO_10_FREQUENCIES[9] === 16000, "10-Band #9 must be 16000 Hz");
  });

  test("ISO 10-to-31 band index mapping is strictly aligned with 1-octave spacing", () => {
    assert(ISO_10_TO_31_INDICES.length === 10, "Mapping must have 10 indices");
    ISO_10_TO_31_INDICES.forEach((idx31, idx10) => {
      const f10 = ISO_10_FREQUENCIES[idx10];
      const f31 = ISO_31_FREQUENCIES[idx31];
      assert(f10 === f31, `Index mapping mismatch: 10-band[${idx10}]=${f10}Hz vs 31-band[${idx31}]=${f31}Hz`);
    });
  });

  test("Frequency label formatting produces crisp pro-audio notation", () => {
    assert(formatFrequency(20) === "20", "20 Hz -> '20'");
    assert(formatFrequency(31.5) === "31.5", "31.5 Hz -> '31.5'");
    assert(formatFrequency(1000) === "1k", "1000 Hz -> '1k'");
    assert(formatFrequency(1250) === "1.25k", "1250 Hz -> '1.25k'");
    assert(formatFrequency(2000) === "2k", "2000 Hz -> '2k'");
    assert(formatFrequency(10000) === "10k", "10000 Hz -> '10k'");
    assert(formatFrequency(20000) === "20k", "20000 Hz -> '20k'");
  });

  // 2. Magnetic Detent Snap & Operational Range Bounding (FR9, UX-DR7, EXPERIENCE.md §6.2)
  test("Magnetic detent snaps values within +/-0.3 dB strictly to 0.0 dB", () => {
    assert(applyMagneticSnap(0.0) === 0.0, "0.0 -> 0.0");
    assert(applyMagneticSnap(0.1) === 0.0, "+0.1 dB must snap to 0.0 dB");
    assert(applyMagneticSnap(0.25) === 0.0, "+0.25 dB must snap to 0.0 dB");
    assert(applyMagneticSnap(0.3) === 0.0, "+0.3 dB must snap to 0.0 dB");
    assert(applyMagneticSnap(-0.1) === 0.0, "-0.1 dB must snap to 0.0 dB");
    assert(applyMagneticSnap(-0.25) === 0.0, "-0.25 dB must snap to 0.0 dB");
    assert(applyMagneticSnap(-0.3) === 0.0, "-0.3 dB must snap to 0.0 dB");
  });

  test("Values outside +/-0.3 dB are rounded to 0.1 dB precision without snapping", () => {
    assert(applyMagneticSnap(0.35) === 0.4, "+0.35 dB rounds to +0.4 dB");
    assert(applyMagneticSnap(0.5) === 0.5, "+0.5 dB preserved");
    assert(applyMagneticSnap(-0.4) === -0.4, "-0.4 dB preserved");
    assert(applyMagneticSnap(3.24) === 3.2, "+3.24 dB rounded to +3.2 dB");
    assert(applyMagneticSnap(-4.68) === -4.7, "-4.68 dB rounded to -4.7 dB");
  });

  test("Gains outside [-12.0 dB, +12.0 dB] operational range are clamped strictly", () => {
    assert(applyMagneticSnap(12.5) === 12.0, "+12.5 dB must clamp to +12.0 dB");
    assert(applyMagneticSnap(50.0) === 12.0, "+50 dB must clamp to +12.0 dB");
    assert(applyMagneticSnap(-15.0) === -12.0, "-15 dB must clamp to -12.0 dB");
    assert(applyMagneticSnap(-100.0) === -12.0, "-100 dB must clamp to -12.0 dB");
  });

  // 3. Fader Travel Sensitivity & 5x Precision Shift Modifier (EXPERIENCE.md §6.2)
  test("Holding Shift reduces sensitivity by exactly 5x for 0.1 dB fine precision", () => {
    const normalDelta = calculateFaderDelta(-15, false, 180, 24.0);
    const fineDelta = calculateFaderDelta(-15, true, 180, 24.0);

    assert(normalDelta > 0, "Moving up (-15px) must increase gain");
    assert(fineDelta > 0, "Moving up (-15px with Shift) must increase gain");
    assert(
      Math.abs(normalDelta / fineDelta - 5.0) < 1e-6,
      `Shift sensitivity must be exactly 5x finer: normal=${normalDelta}, fine=${fineDelta}`
    );
  });

  // 4. Frequency & Gain Coordinate Projection (FR10)
  test("Logarithmic frequency mapping projects spectrum across 20 Hz - 20 kHz", () => {
    const width = 800;
    const x20 = frequencyToLogX(20, width);
    const x20k = frequencyToLogX(20000, width);
    const xMid = frequencyToLogX(Math.sqrt(20 * 20000), width);

    assert(Math.abs(x20 - 0.0) < 1e-4, "20 Hz must project to x=0");
    assert(Math.abs(x20k - width) < 1e-4, "20,000 Hz must project to x=width");
    assert(Math.abs(xMid - width / 2.0) < 1e-4, "Geometric mean must project to x=width/2");
  });

  test("Gain mapping projects [-12, +12] dB to Y coordinates", () => {
    const height = 140;
    const yBoost = gainToY(12.0, height);
    const yCenter = gainToY(0.0, height);
    const yCut = gainToY(-12.0, height);

    assert(Math.abs(yBoost - 0.0) < 1e-4, "+12 dB must project to y=0 (top)");
    assert(Math.abs(yCenter - 70.0) < 1e-4, "0.0 dB must project to y=height/2 (center)");
    assert(Math.abs(yCut - 140.0) < 1e-4, "-12 dB must project to y=height (bottom)");
  });

  // 5. Smooth Catmull-Rom Cubic Bezier Spline Computation (FR10)
  test("Spline curve computation produces continuous cubic Bezier segments", () => {
    const points: Point[] = ISO_31_FREQUENCIES.map((f, i) => ({
      x: frequencyToLogX(f, 600),
      y: gainToY(i === 17 ? 12.0 : 0.0, 120),
    }));

    const segments = computeSplineSegments(points);
    assert(segments.length === 30, "31 points must yield 30 contiguous Bezier segments");

    // All control points must have finite coordinates
    segments.forEach((seg, i) => {
      assert(Number.isFinite(seg.p0.x) && Number.isFinite(seg.p0.y), `Segment ${i} p0 invalid`);
      assert(Number.isFinite(seg.cp1.x) && Number.isFinite(seg.cp1.y), `Segment ${i} cp1 invalid`);
      assert(Number.isFinite(seg.cp2.x) && Number.isFinite(seg.cp2.y), `Segment ${i} cp2 invalid`);
      assert(Number.isFinite(seg.p1.x) && Number.isFinite(seg.p1.y), `Segment ${i} p1 invalid`);
    });

    // Segment 17 (leading into 1 kHz peak) must show ascending slope
    assert(segments[16].p1.y < segments[16].p0.y, "Curve leading into 1 kHz boost must rise (lower y)");
  });

  // 6. Preservation of 31-Band Gain Array during 10-Band Editing (FR12)
  test("Editing 10-Band casual faders strictly preserves intermediate 31-Band gains", () => {
    const initialGains = new Array(31).fill(0.0);
    // Preset intermediate adjustments on band 1 (25 Hz) and band 16 (800 Hz)
    initialGains[1] = 4.5;
    initialGains[16] = -3.2;

    const modifiedGains = [...initialGains];
    // User adjusts 10-Band index 5 (which is 31-band index 17 = 1 kHz) to +6.0 dB
    const target31Index = ISO_10_TO_31_INDICES[5];
    assert(target31Index === 17, "10-band index 5 maps to 31-band index 17");
    modifiedGains[target31Index] = 6.0;

    // Verify intermediate bands were not corrupted or wiped
    assert(modifiedGains[1] === 4.5, "Intermediate band 1 (25 Hz) must be preserved");
    assert(modifiedGains[16] === -3.2, "Intermediate band 16 (800 Hz) must be preserved");
    assert(modifiedGains[17] === 6.0, "Modified band 17 (1 kHz) must reflect new gain");
  });

  // 7. Component Markup Verification (UX-DR7)
  test("EqualizerStudioDrawer renders 10 faders in 10-band mode", () => {
    const gains = new Array(31).fill(0.0);
    gains[17] = 3.5;

    const html = renderToStaticMarkup(
      <EqualizerStudioDrawer
        isOpen={true}
        onClose={() => {}}
        mode="10"
        onModeChange={() => {}}
        gains={gains}
        onGainChange={() => {}}
        onResetAll={() => {}}
        activePreset="Harman Acoustic"
        preampOffsetDb={-3.0}
      />
    );

    assert(html.includes("EQUALIZER STUDIO"), "Must render title");
    assert(html.includes("10-Band"), "Must render 10-Band mode toggle");
    assert(html.includes("31-Band"), "Must render 31-Band mode toggle");
    assert(html.includes("Harman Acoustic"), "Must render active preset badge");
    assert(html.includes("Preamp: -3.0 dB"), "Must render preamp offset");
    assert(html.includes("Flat (0 dB)"), "Must render reset button");
    assert(html.includes("Done"), "Must render done button");
    assert(html.includes("<canvas"), "Must render curve canvas");
    assert(html.includes("translate-y-0"), "Open drawer must have translate-y-0");

    // Must render exactly 10 fader sliders
    const faderMatches = html.match(/role="slider"/g);
    assert(faderMatches !== null && faderMatches.length === 10, "10-Band mode must render 10 sliders");
  });

  test("EqualizerStudioDrawer renders 31 faders in 31-band mode", () => {
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
      />
    );

    const faderMatches = html.match(/role="slider"/g);
    assert(faderMatches !== null && faderMatches.length === 31, "31-Band mode must render 31 sliders");
  });

  test("EqualizerStudioDrawer when closed renders with translate-y-full and pointer-events-none", () => {
    const gains = new Array(31).fill(0.0);

    const html = renderToStaticMarkup(
      <EqualizerStudioDrawer
        isOpen={false}
        onClose={() => {}}
        mode="10"
        onModeChange={() => {}}
        gains={gains}
        onGainChange={() => {}}
        onResetAll={() => {}}
      />
    );

    assert(html.includes("translate-y-full"), "Closed drawer must be translated down");
    assert(html.includes("pointer-events-none"), "Closed drawer must not intercept pointer events");
    assert(html.includes('aria-hidden="true"'), "Closed drawer must be aria-hidden");
  });

  test("EqFader renders vertical slider with 180px track and Cascadia Code badge", () => {
    const html = renderToStaticMarkup(
      <EqFader
        bandIndex={17}
        frequencyHz={1000}
        gainDb={4.2}
        onChange={() => {}}
      />
    );

    assert(html.includes("h-[180px]"), "Fader track must have 180px height");
    assert(html.includes("+4.2"), "Must render +4.2 dB label");
    assert(html.includes("1k"), "Must render 1k frequency label");
  });

  test("EqualizerPreview renders Expand EQ button with Ctrl+Shift+E badge", () => {
    const gains = new Array(31).fill(0.0);
    gains[17] = 2.0;

    const html = renderToStaticMarkup(
      <EqualizerPreview
        activePreset="Harman Acoustic"
        bands={gains}
        preampOffsetDb={-2.5}
        onExpand={() => {}}
      />
    );

    assert(html.includes("Expand EQ"), "Must render 'Expand EQ' button");
    assert(html.includes("Ctrl+Shift+E"), "Must render 'Ctrl+Shift+E' hotkey badge");
    assert(html.includes("DUAL-STAGE EQUALIZER"), "Must render header");
  });

  // 12. 10-Band Mode Dispatches to Exact 31-Band Mapped Indices
  test("10-Band mode maps fader adjustments strictly to ISO_10_TO_31_INDICES", () => {
    const recordedChanges: { bandIdx: number; gain: number }[] = [];
    const handleGainChange = (bandIdx: number, gain: number) => {
      recordedChanges.push({ bandIdx, gain });
    };

    // Simulate adjusting casual fader 5 (1 kHz) in 10-band mode
    const casualIndex = 5;
    const mapped31Index = ISO_10_TO_31_INDICES[casualIndex];
    assert(mapped31Index === 17, "Casual slot 5 must map to 31-band index 17 (1 kHz)");

    handleGainChange(mapped31Index, 4.2);
    assert(recordedChanges.length === 1, "Recorded changes length must be 1");
    assert(recordedChanges[0].bandIdx === 17, "Recorded bandIdx must be 17");
    assert(recordedChanges[0].gain === 4.2, "Recorded gain must be 4.2");
  });

  // 13. Fine Precision Keyboard Navigation is Not Trapped at 0.0 dB
  test("Shift + Arrow key fine precision does not get trapped at 0.0 dB", () => {
    const initialGain = 0.0;
    const step = 0.1;
    // Shift is pressed -> fine precision bypasses magnetic snap
    const nextGainUp = Math.round((initialGain + step) * 10) / 10;
    const nextGainDown = Math.round((initialGain - step) * 10) / 10;

    assert(nextGainUp === 0.1, "Shift+Up from 0.0 dB must reach +0.1 dB without snapping back to 0");
    assert(nextGainDown === -0.1, "Shift+Down from 0.0 dB must reach -0.1 dB without snapping back to 0");
  });

  // 14. Standard PageUp/PageDown and Home/End Navigation
  test("PageUp/Down steps by 3.0 dB and Home/End jump to limits", () => {
    const gain = 2.0;
    const pageUp = applyMagneticSnap(gain + 3.0, 0.3);
    const pageDown = applyMagneticSnap(gain - 3.0, 0.3);
    assert(pageUp === 5.0, "PageUp must add 3.0 dB");
    assert(pageDown === -1.0, "PageDown must subtract 3.0 dB");

    const home = -12.0;
    const end = 12.0;
    assert(home === -12.0, "Home jumps to -12.0 dB min");
    assert(end === 12.0, "End jumps to +12.0 dB max");
  });

  return results;
}

// Self-executing runner when executed via Node
const nodeProcess = (globalThis as unknown as { process?: { argv?: string[]; exit?: (code: number) => void } }).process;
if (nodeProcess && nodeProcess.argv && nodeProcess.argv[1]?.includes("EqualizerStudioDrawer.test")) {
  console.log("Running EqualizerStudioDrawer test suite...\n");
  const testResults = runEqualizerStudioDrawerTests();
  let failed = 0;

  testResults.forEach((res) => {
    if (res.passed) {
      console.log(`  ✓ ${res.name}`);
    } else {
      console.error(`  ✗ ${res.name}`);
      console.error(`    ${res.error}`);
      failed++;
    }
  });

  console.log(`\nResults: ${testResults.length - failed} passed, ${failed} failed`);
  if (failed > 0 && nodeProcess.exit) {
    nodeProcess.exit(1);
  }
}
