import { renderToStaticMarkup } from "react-dom/server";
import {
  PresetSelectorPopover,
  FACTORY_EQ_PRESETS,
  isPresetDirty,
  formatPresetDisplayLabel,
  toPresetSlug,
  validateCustomPresetName,
  EqPreset,
} from "../components/PresetSelectorPopover";
import { EqualizerPreview } from "../components/EqualizerPreview";
import { EqualizerStudioDrawer } from "../components/EqualizerStudioDrawer";

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

export function runPresetSelectorTests(): TestResult[] {
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

  test("1. Provides all 15 required factory genre presets with 31-band curves (FR11)", () => {
    assert(
      FACTORY_EQ_PRESETS.length === 15,
      `Expected 15 factory presets, got ${FACTORY_EQ_PRESETS.length}`
    );
    const required = [
      "Flat",
      "Bass Boost",
      "Treble Boost",
      "Acoustic",
      "Classical",
      "Electronic",
      "Gaming",
      "Hip-Hop",
      "Jazz",
      "Movie",
      "Pop",
      "Rock",
      "Vocal",
      "Lounge",
      "Deep",
    ];
    for (const name of required) {
      const found = FACTORY_EQ_PRESETS.find((p) => p.name === name);
      assert(Boolean(found), `Missing required factory preset: ${name}`);
      assert(found!.is_factory === true, `${name} must have is_factory=true`);
      assert(
        found!.gains_db.length === 31,
        `${name} must have 31 bands, got ${found!.gains_db.length}`
      );
      for (const g of found!.gains_db) {
        assert(
          Number.isFinite(g) && g >= -12.0 && g <= 12.0,
          `Gain ${g} out of bounds in ${name}`
        );
      }
    }
  });

  test("2. Detects dirty preset state when any band deviates by > 0.05 dB (EXPERIENCE.md §5.2)", () => {
    const rock = FACTORY_EQ_PRESETS.find((p) => p.name === "Rock")!;
    const exactCopy = [...rock.gains_db];
    assert(
      !isPresetDirty(exactCopy, rock.gains_db),
      "Exact copy of Rock curve must not be dirty"
    );

    const tinyNoise = [...rock.gains_db];
    tinyNoise[5] += 0.02;
    assert(
      !isPresetDirty(tinyNoise, rock.gains_db),
      "Deviation <= 0.05 dB must not trigger dirty state"
    );

    const modified = [...rock.gains_db];
    modified[10] += 0.5;
    assert(
      isPresetDirty(modified, rock.gains_db),
      "Fader adjustment of +0.5 dB must mark preset as dirty"
    );
  });

  test("3. Formats active preset display label with asterisk when dirty (Rock *)", () => {
    assert(
      formatPresetDisplayLabel("Rock", false) === "Rock",
      "Clean preset must not have asterisk"
    );
    assert(
      formatPresetDisplayLabel("Rock", true) === "Rock *",
      "Modified preset must display 'Rock *'"
    );
    assert(
      formatPresetDisplayLabel("", true) === "Flat *",
      "Empty preset name falls back to 'Flat *'"
    );
  });

  test("4. Sanitizes custom preset slugs and validates against factory and reserved names", () => {
    assert(
      toPresetSlug("  My Late Night EQ! ") === "my-late-night-eq",
      "Should slugify spaces and punctuation"
    );
    assert(
      toPresetSlug("../../etc/passwd") === "etc-passwd",
      "Should strip path traversal tokens"
    );

    // Reject empty or non-alphanumeric
    assert(
      !validateCustomPresetName("   ", FACTORY_EQ_PRESETS).valid,
      "Empty name must be invalid"
    );
    assert(
      !validateCustomPresetName("...///", FACTORY_EQ_PRESETS).valid,
      "Punctuation-only name must be invalid"
    );

    // Reject factory collisions
    assert(
      !validateCustomPresetName("Rock", FACTORY_EQ_PRESETS).valid,
      "Factory name 'Rock' must be rejected"
    );
    assert(
      !validateCustomPresetName("bass boost", FACTORY_EQ_PRESETS).valid,
      "Case-insensitive factory name 'bass boost' must be rejected"
    );
    assert(
      !validateCustomPresetName("CON", FACTORY_EQ_PRESETS).valid,
      "Windows reserved name 'CON' must be rejected"
    );

    // Valid custom name
    assert(
      validateCustomPresetName("Warm Late Night", FACTORY_EQ_PRESETS).valid,
      "'Warm Late Night' must be valid"
    );
  });

  test("5. PresetSelectorPopover renders all 15 factory presets, custom presets, and dirty badge", () => {
    const customPreset: EqPreset = {
      id: "my-custom-curve",
      name: "My Custom Curve",
      category: "Custom",
      is_factory: false,
      gains_db: new Array(31).fill(1.5),
    };
    const allPresets = [...FACTORY_EQ_PRESETS, customPreset];

    const html = renderToStaticMarkup(
      <PresetSelectorPopover
        isOpen={true}
        onClose={() => {}}
        presets={allPresets}
        activePresetName="Rock"
        currentGains={new Array(31).fill(2.0)}
        isDirty={true}
        onSelectPreset={() => {}}
        onSaveCustomPreset={() => {}}
        onRenameCustomPreset={() => {}}
        onDeleteCustomPreset={() => {}}
      />
    );

    assert(html.includes("Rock *"), "Popover must display 'Rock *' when dirty");
    assert(html.includes("Modified"), "Popover must display 'Modified' badge when dirty");
    assert(
      html.includes("Save As Custom Preset"),
      "Popover must include 'Save As Custom Preset' button"
    );
    assert(
      html.includes("My Custom Curve"),
      "Popover must render custom preset entry"
    );
    assert(
      html.includes("Rename My Custom Curve") && html.includes("Delete My Custom Curve"),
      "Popover must render Rename and Delete actions for custom presets"
    );
    for (const fp of FACTORY_EQ_PRESETS) {
      assert(
        html.includes(fp.name),
        `Popover must render factory preset ${fp.name}`
      );
    }
  });

  test("6. EqualizerPreview and EqualizerStudioDrawer display dirty asterisk indicator (Rock *)", () => {
    const previewHtml = renderToStaticMarkup(
      <EqualizerPreview
        activePreset="Rock"
        isPresetDirty={true}
        bands={new Array(31).fill(0)}
        preampOffsetDb={-3.5}
      />
    );
    assert(
      previewHtml.includes("Rock *"),
      "EqualizerPreview must display 'Rock *' when isPresetDirty is true"
    );

    const drawerHtml = renderToStaticMarkup(
      <EqualizerStudioDrawer
        isOpen={true}
        onClose={() => {}}
        mode="31"
        onModeChange={() => {}}
        gains={new Array(31).fill(0)}
        onGainChange={() => {}}
        onResetAll={() => {}}
        activePreset="Gaming"
        isPresetDirty={true}
      />
    );
    assert(
      drawerHtml.includes("Gaming *"),
      "EqualizerStudioDrawer must display 'Gaming *' when isPresetDirty is true"
    );
  });

  test("7. Custom preset rename validation prevents colliding with another custom preset", () => {
    const c1: EqPreset = {
      id: "custom-one",
      name: "Custom One",
      category: "Custom",
      is_factory: false,
      gains_db: new Array(31).fill(0),
    };
    const c2: EqPreset = {
      id: "custom-two",
      name: "Custom Two",
      category: "Custom",
      is_factory: false,
      gains_db: new Array(31).fill(1),
    };
    const list = [...FACTORY_EQ_PRESETS, c1, c2];

    assert(
      !validateCustomPresetName("Custom Two", list, "custom-one").valid,
      "Renaming custom-one to Custom Two must be rejected"
    );
    assert(
      validateCustomPresetName("Custom ONE", list, "custom-one").valid,
      "Renaming custom-one to change casing of its own name must be allowed"
    );
  });

  test("8. PresetSelectorPopover hides cleanly with aria-hidden when closed", () => {
    const closedHtml = renderToStaticMarkup(
      <PresetSelectorPopover
        isOpen={false}
        onClose={() => {}}
        presets={FACTORY_EQ_PRESETS}
        activePresetName="Flat"
        currentGains={new Array(31).fill(0)}
        isDirty={false}
        onSelectPreset={() => {}}
        onSaveCustomPreset={() => {}}
        onRenameCustomPreset={() => {}}
        onDeleteCustomPreset={() => {}}
      />
    );
    assert(
      closedHtml.includes('aria-hidden="true"') && closedHtml.includes("display:none"),
      "Closed PresetSelectorPopover must have aria-hidden=true and display:none"
    );
  });

  return results;
}
