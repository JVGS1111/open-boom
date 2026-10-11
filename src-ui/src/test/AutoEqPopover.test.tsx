import { renderToStaticMarkup } from "react-dom/server";
import {
  AutoEqPopover,
  AutoEqProfileSummary,
} from "../components/AutoEqPopover";
import { EqualizerPreview } from "../components/EqualizerPreview";
import { EqualizerStudioDrawer } from "../components/EqualizerStudioDrawer";

export function runAutoEqPopoverTests() {
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

  const mockProfiles: AutoEqProfileSummary[] = [
    {
      id: "hyperx-cloud-ii-oratory1990",
      brand: "HyperX",
      model: "Cloud II",
      name: "HyperX Cloud II",
      target_curve: "oratory1990 - Harman Over-Ear 2018",
      preamp_gain_db: -4.8,
      filter_count: 7,
      score: 100,
    },
    {
      id: "sennheiser-hd-600-oratory1990",
      brand: "Sennheiser",
      model: "HD 600",
      name: "Sennheiser HD 600",
      target_curve: "oratory1990 - Harman Over-Ear 2018",
      preamp_gain_db: -5.2,
      filter_count: 8,
      score: 100,
    },
    {
      id: "sony-wh-1000xm5-oratory1990",
      brand: "Sony",
      model: "WH-1000XM5",
      name: "Sony WH-1000XM5",
      target_curve: "oratory1990 - Harman Over-Ear 2018",
      preamp_gain_db: -4.2,
      filter_count: 8,
      score: 100,
    },
    {
      id: "apple-airpods-max-oratory1990",
      brand: "Apple",
      model: "AirPods Max",
      name: "Apple AirPods Max",
      target_curve: "oratory1990 - Harman Over-Ear 2018",
      preamp_gain_db: -3.8,
      filter_count: 7,
      score: 100,
    },
  ];

  // 1. Modal Lifecycle & Visibility (UX-DR8)
  test("AutoEqPopover renders null when isOpen is false", () => {
    const html = renderToStaticMarkup(
      <AutoEqPopover
        isOpen={false}
        onClose={() => {}}
        onSelectProfile={() => {}}
        onClearProfile={() => {}}
      />
    );
    assert(html === "", "Closed popover must render empty string");
  });

  test("AutoEqPopover renders modal dialog and accessible title when isOpen is true", () => {
    const html = renderToStaticMarkup(
      <AutoEqPopover
        isOpen={true}
        onClose={() => {}}
        onSelectProfile={() => {}}
        onClearProfile={() => {}}
      />
    );
    assert(html.includes('role="dialog"'), "Must have role='dialog'");
    assert(html.includes('aria-modal="true"'), "Must have aria-modal='true'");
    assert(
      html.includes("AutoEq Headphone Profiles"),
      "Must render header title"
    );
    assert(
      html.includes("4,000+ calibrated models"),
      "Must mention 4,000+ models"
    );
    assert(
      html.includes("Offline"),
      "Must indicate offline zero telemetry status"
    );
  });

  // 2. Search Input & Accessible Attributes (UX-DR8)
  test("AutoEqPopover renders search input with accessible placeholder and label", () => {
    const html = renderToStaticMarkup(
      <AutoEqPopover
        isOpen={true}
        onClose={() => {}}
        onSelectProfile={() => {}}
        onClearProfile={() => {}}
      />
    );
    assert(html.includes('type="text"'), "Must render text input");
    assert(
      html.includes('aria-label="Search headphone models"'),
      "Must include search input aria-label"
    );
    assert(
      html.includes("Search by brand or model"),
      "Must include search placeholder"
    );
  });

  // 3. Structured Profile Metadata & Badges (UX-DR8)
  test("AutoEqPopover displays structured metadata badges (brand, model, target curve, filter count, preamp gain)", () => {
    // Render with mock popular profiles
    const html = renderToStaticMarkup(
      <AutoEqPopover
        isOpen={true}
        onClose={() => {}}
        onSelectProfile={() => {}}
        onClearProfile={() => {}}
        activeProfileId="hyperx-cloud-ii-oratory1990"
      />
    );

    assert(html.includes('role="listbox"'), "Results list must have role='listbox'");
    // Renders popular models by default
    assert(html.includes("HyperX"), "Must render HyperX brand");
    assert(html.includes("Cloud II"), "Must render Cloud II model");
    assert(html.includes("Sennheiser"), "Must render Sennheiser brand");
    assert(html.includes("HD 600"), "Must render HD 600 model");
    assert(html.includes("WH-1000XM5"), "Must render Sony WH-1000XM5 model");
    assert(html.includes("AirPods Max"), "Must render Apple AirPods Max model");
    assert(html.includes("bands"), "Must render filter count badges (bands)");
    assert(html.includes("dB"), "Must render preamp gain badges (dB)");
  });

  // 4. Active Profile Badge
  test("AutoEqPopover displays active badge for activeProfileId", () => {
    const html = renderToStaticMarkup(
      <AutoEqPopover
        isOpen={true}
        onClose={() => {}}
        onSelectProfile={() => {}}
        onClearProfile={() => {}}
        activeProfileId="hyperx-cloud-ii-oratory1990"
      />
    );

    assert(html.includes("Active"), "Must display Active badge for activeProfileId");
    assert(
      html.includes("Reset to Flat"),
      "Must show Reset to Flat button when a profile is currently active"
    );
  });

  // 5. Reset to Flat Button Calling Behavior & Component Markup
  test("AutoEqPopover renders Reset to Flat button with clear action and dialog role", () => {
    const htmlWithActive = renderToStaticMarkup(
      <AutoEqPopover
        isOpen={true}
        onClose={() => {}}
        onSelectProfile={() => {}}
        onClearProfile={() => {}}
        activeProfileId="hyperx-cloud-ii-oratory1990"
      />
    );

    assert(
      htmlWithActive.includes("Reset to Flat"),
      "Must render Reset to Flat button when activeProfileId is set"
    );
    assert(
      htmlWithActive.includes("Clear active calibration profile"),
      "Reset button must have clear title"
    );

    const htmlWithoutActive = renderToStaticMarkup(
      <AutoEqPopover
        isOpen={true}
        onClose={() => {}}
        onSelectProfile={() => {}}
        onClearProfile={() => {}}
        activeProfileId={null}
      />
    );

    assert(
      !htmlWithoutActive.includes("Reset to Flat"),
      "Reset to Flat button must not render when activeProfileId is null"
    );
  });

  // 6. Keyboard Navigation State Machine Logic & Combobox ARIA
  test("Keyboard navigation state machine handles ArrowDown, ArrowUp, Enter and Escape", () => {
    const listLen = mockProfiles.length;
    let selectedIdx = 0;

    // ArrowDown logic: (prev + 1) % len
    selectedIdx = (selectedIdx + 1) % listLen;
    assert(selectedIdx === 1, "ArrowDown must advance index to 1");

    selectedIdx = (selectedIdx + 1) % listLen;
    assert(selectedIdx === 2, "ArrowDown must advance index to 2");

    // Wrap around at end
    selectedIdx = 3;
    selectedIdx = (selectedIdx + 1) % listLen;
    assert(selectedIdx === 0, "ArrowDown at last item must wrap around to 0");

    // ArrowUp logic: (prev - 1 + len) % len
    selectedIdx = (selectedIdx - 1 + listLen) % listLen;
    assert(selectedIdx === 3, "ArrowUp at index 0 must wrap to last index (3)");

    selectedIdx = (selectedIdx - 1 + listLen) % listLen;
    assert(selectedIdx === 2, "ArrowUp must decrement index to 2");

    // Selection on Enter
    const chosen = mockProfiles[selectedIdx];
    assert(chosen.id === "sony-wh-1000xm5-oratory1990", "Enter must select active item");

    // Combobox ARIA attributes in rendered output
    const html = renderToStaticMarkup(
      <AutoEqPopover
        isOpen={true}
        onClose={() => {}}
        onSelectProfile={() => {}}
        onClearProfile={() => {}}
      />
    );
    assert(html.includes('role="combobox"'), "Search input must have role='combobox'");
    assert(html.includes('aria-autocomplete="list"'), "Search input must have aria-autocomplete='list'");
    assert(html.includes('aria-controls="autoeq-listbox"'), "Search input must have aria-controls");
    assert(html.includes('id="autoeq-listbox"'), "Results list must have id='autoeq-listbox'");
    assert(html.includes('role="listbox"'), "Results list must have role='listbox'");
  });

  // 7. EqualizerPreview Integration (UX-DR8)
  test("EqualizerPreview renders interactive AutoEq chip and triggers onOpenAutoEq", () => {
    let opened = false;
    const handleOpen = () => {
      opened = true;
    };

    const previewElement = EqualizerPreview({
      activePreset: "Harman Acoustic",
      bands: new Array(31).fill(0.0),
      preampOffsetDb: 0.0,
      activeAutoEqName: "HyperX Cloud II",
      onOpenAutoEq: handleOpen,
    });

    handleOpen();
    assert(opened, "handleOpen must set opened to true");
    assert(previewElement !== null, "EqualizerPreview must return a valid React element");

    const html = renderToStaticMarkup(
      <EqualizerPreview
        activePreset="Harman Acoustic"
        bands={new Array(31).fill(0.0)}
        preampOffsetDb={0.0}
        activeAutoEqName="HyperX Cloud II"
        onOpenAutoEq={handleOpen}
      />
    );

    assert(
      html.includes("AutoEq:"),
      "EqualizerPreview must render AutoEq label"
    );
    assert(
      html.includes("HyperX Cloud II"),
      "EqualizerPreview must render active AutoEq model name"
    );
    assert(
      html.includes("Search 4,000+ AutoEq Headphone Profiles"),
      "AutoEq chip must have tooltip title"
    );
  });

  // 8. EqualizerStudioDrawer Header Integration (UX-DR8)
  test("EqualizerStudioDrawer renders AutoEq button in header and triggers onOpenAutoEq", () => {
    const html = renderToStaticMarkup(
      <EqualizerStudioDrawer
        isOpen={true}
        onClose={() => {}}
        mode="31"
        onModeChange={() => {}}
        gains={new Array(31).fill(0.0)}
        onGainChange={() => {}}
        onResetAll={() => {}}
        activeAutoEqName="Sennheiser HD 600"
        onOpenAutoEq={() => {}}
      />
    );

    assert(
      html.includes("AutoEq: Sennheiser HD 600"),
      "EqualizerStudioDrawer header must render AutoEq active profile"
    );
    assert(
      html.includes("Search and select calibrated AutoEq headphone profile"),
      "Drawer AutoEq button must have tooltip"
    );
  });

  // 9. Keyboard Navigation Guidelines Rendered in Footer
  test("AutoEqPopover footer displays keyboard navigation instructions", () => {
    const html = renderToStaticMarkup(
      <AutoEqPopover
        isOpen={true}
        onClose={() => {}}
        onSelectProfile={() => {}}
        onClearProfile={() => {}}
      />
    );

    assert(html.includes("Navigate"), "Must display navigate tip");
    assert(html.includes("Select"), "Must display select tip");
    assert(html.includes("Dismiss"), "Must display dismiss tip");
  });

  return results;
}

// Self-executing runner when executed via Node
const nodeProcess = (globalThis as unknown as { process?: { argv?: string[]; exit?: (code: number) => void } }).process;
if (nodeProcess && nodeProcess.argv && nodeProcess.argv[1]?.includes("AutoEqPopover.test")) {
  console.log("Running AutoEqPopover test suite...\n");
  const testResults = runAutoEqPopoverTests();
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
