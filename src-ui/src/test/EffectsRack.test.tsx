import { renderToStaticMarkup } from "react-dom/server";
import { EffectsRackStrip, EffectModule } from "../components/EffectsRackStrip";

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

export function runEffectsRackTests(): TestResult[] {
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

  const sampleModules: EffectModule[] = [
    {
      id: 0,
      name: "Ambience",
      description: "FDN Reverb Matrix",
      enabled: true,
      amount: 0.35,
    },
    {
      id: 1,
      name: "Fidelity",
      description: "Harmonic Clarity Exciter",
      enabled: true,
      amount: 0.45,
    },
    {
      id: 2,
      name: "Night Mode",
      description: "Dynamic Range Soft-Knee",
      enabled: false,
      amount: 0.50,
    },
    {
      id: 3,
      name: "Spatial",
      description: "Mid/Side Stereo Crossfeed",
      enabled: true,
      amount: 0.60,
    },
    {
      id: 4,
      name: "Pitch",
      description: "Pitch Shifter (-12..+12 st)",
      enabled: false,
      amount: 0.00,
      isComingSoon: true,
    },
  ];

  test("1. Renders all 5 modular cards in a 5-column grid (UX-DR6, FR20)", () => {
    const html = renderToStaticMarkup(
      <EffectsRackStrip
        modules={sampleModules}
        onToggleModule={() => {}}
        onAmountChange={() => {}}
      />
    );

    assert(html.includes("grid-cols-5"), "Must use grid-cols-5 layout");
    assert(html.includes("Ambience"), "Must contain Ambience card");
    assert(html.includes("Fidelity"), "Must contain Fidelity card");
    assert(html.includes("Night Mode"), "Must contain Night Mode card");
    assert(html.includes("Spatial"), "Must contain Spatial card");
    assert(html.includes("Pitch"), "Must contain Pitch card");
  });

  test("2. Active cards render with cyan borders and illuminated glow (DESIGN.md)", () => {
    const html = renderToStaticMarkup(
      <EffectsRackStrip
        modules={sampleModules}
        onToggleModule={() => {}}
        onAmountChange={() => {}}
      />
    );

    assert(
      html.includes("border-accent-cyan/60"),
      "Active modules must have cyan border"
    );
    assert(
      html.includes("shadow-cyan-glow-sm"),
      "Active modules must have cyan glow"
    );
  });

  test("3. Inactive/bypassed cards render with subtle border and dimmed state", () => {
    const html = renderToStaticMarkup(
      <EffectsRackStrip
        modules={sampleModules}
        onToggleModule={() => {}}
        onAmountChange={() => {}}
      />
    );

    assert(
      html.includes("border-border-subtle"),
      "Inactive cards must have subtle border"
    );
    assert(
      html.includes("text-text-muted"),
      "Inactive cards must display dimmed text"
    );
  });

  test("4. Pitch card displays Phase 2 'Soon' badge and semitone indicators (FR20, PRD §6.2)", () => {
    const html = renderToStaticMarkup(
      <EffectsRackStrip
        modules={sampleModules}
        onToggleModule={() => {}}
        onAmountChange={() => {}}
      />
    );

    assert(
      html.includes("Soon"),
      "Pitch card must render 'Soon' badge"
    );
    assert(
      html.includes("Phase 2"),
      "Pitch card must render 'Phase 2' placeholder"
    );
    assert(
      html.includes("0 st"),
      "Pitch card must render default 0 st value"
    );
  });

  test("5. Mini-sliders render with accessible ARIA slider roles and values", () => {
    const html = renderToStaticMarkup(
      <EffectsRackStrip
        modules={sampleModules}
        onToggleModule={() => {}}
        onAmountChange={() => {}}
      />
    );

    assert(html.includes('role="slider"'), "Sliders must expose role='slider'");
    assert(html.includes('aria-valuenow="35"'), "Ambience must expose 35%");
    assert(html.includes('aria-valuenow="45"'), "Fidelity must expose 45%");
    assert(html.includes('aria-valuenow="60"'), "Spatial must expose 60%");
  });

  test("6. Monospace percentage readouts display exact formatted values", () => {
    const html = renderToStaticMarkup(
      <EffectsRackStrip
        modules={sampleModules}
        onToggleModule={() => {}}
        onAmountChange={() => {}}
      />
    );

    assert(html.includes("35%"), "Must render 35% text");
    assert(html.includes("45%"), "Must render 45% text");
    assert(html.includes("50%"), "Must render 50% text");
    assert(html.includes("60%"), "Must render 60% text");
  });

  test("7. Power switches expose switch role and accessible labels", () => {
    const html = renderToStaticMarkup(
      <EffectsRackStrip
        modules={sampleModules}
        onToggleModule={() => {}}
        onAmountChange={() => {}}
      />
    );

    assert(html.includes('role="switch"'), "Switches must expose role='switch'");
    assert(
      html.includes('aria-label="Toggle Ambience"'),
      "Must have accessible label for Ambience switch"
    );
    assert(
      html.includes('aria-checked="true"'),
      "Active switches must have aria-checked='true'"
    );
    assert(
      html.includes('aria-checked="false"'),
      "Inactive switches must have aria-checked='false'"
    );
  });

  test("8. Disabled mode disables interactions and dims rack", () => {
    const html = renderToStaticMarkup(
      <EffectsRackStrip
        modules={sampleModules}
        onToggleModule={() => {}}
        onAmountChange={() => {}}
        disabled={true}
      />
    );

    assert(
      html.includes("opacity-50 pointer-events-none"),
      "Rack must be dimmed and non-interactive when disabled"
    );
  });

  test("9. Tooltips reflect audio technical descriptions (EXPERIENCE.md §3)", () => {
    const html = renderToStaticMarkup(
      <EffectsRackStrip
        modules={sampleModules}
        onToggleModule={() => {}}
        onAmountChange={() => {}}
      />
    );

    assert(
      html.includes("Ambience: FDN Reverb Matrix"),
      "Must include tooltip for Ambience"
    );
    assert(
      html.includes("Fidelity: Harmonic Clarity Exciter"),
      "Must include tooltip for Fidelity"
    );
    assert(
      html.includes("Night Mode: Dynamic Range Soft-Knee"),
      "Must include tooltip for Night Mode"
    );
  });

  return results;
}
