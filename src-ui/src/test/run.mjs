import esbuild from "esbuild";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const testFiles = [
  {
    name: "EqualizerStudioDrawer & Spline Curve Test Suite",
    entry: path.join(__dirname, "EqualizerStudioDrawer.test.tsx"),
    runnerName: "runEqualizerStudioDrawerTests",
  },
  {
    name: "AutoEqPopover & Offline Headphone Search Test Suite",
    entry: path.join(__dirname, "AutoEqPopover.test.tsx"),
    runnerName: "runAutoEqPopoverTests",
  },
  {
    name: "AutoEq Parametric Stage & Preamp Auto-Gain Test Suite",
    entry: path.join(__dirname, "AutoEqParametric.test.tsx"),
    runnerName: "runAutoEqParametricTests",
  },
  {
    name: "Genre Factory Presets & Custom Preset Persistence Test Suite",
    entry: path.join(__dirname, "PresetSelector.test.tsx"),
    runnerName: "runPresetSelectorTests",
  },
  {
    name: "5-Effect Quick Rack Strip UI Component Test Suite",
    entry: path.join(__dirname, "EffectsRack.test.tsx"),
    runnerName: "runEffectsRackTests",
  },
  {
    name: "Per-Application Mixer Drawer & Process Faders Test Suite",
    entry: path.join(__dirname, "AppMixer.test.tsx"),
    runnerName: "runAppMixerTests",
  },
];

async function main() {
  let totalPassed = 0;
  let totalFailed = 0;

  for (const suite of testFiles) {
    const tempOut = path.join(__dirname, `.test-${path.basename(suite.entry, ".tsx")}.mjs`);
    try {
      await esbuild.build({
        entryPoints: [suite.entry],
        outfile: tempOut,
        bundle: true,
        format: "esm",
        platform: "node",
        external: [
          "react",
          "react-dom",
          "react-dom/server",
          "lucide-react",
          "@tauri-apps/api",
          "@tauri-apps/api/*",
        ],
      });

      const mod = await import(`file://${tempOut.replace(/\\/g, "/")}`);
      const results = mod[suite.runnerName]();

      console.log("\n=======================================================");
      console.log(` ${suite.name}`);
      console.log("=======================================================\n");

      let suitePassed = 0;
      let suiteFailed = 0;

      results.forEach((r) => {
        if (r.passed) {
          console.log(`  ✓ ${r.name}`);
          suitePassed++;
          totalPassed++;
        } else {
          console.error(`  ✗ ${r.name}`);
          console.error(`    Error: ${r.error}`);
          suiteFailed++;
          totalFailed++;
        }
      });

      console.log("\n-------------------------------------------------------");
      console.log(` Suite Summary: ${suitePassed} passed, ${suiteFailed} failed (${results.length} total)`);
      console.log("=======================================================\n");
    } finally {
      if (fs.existsSync(tempOut)) {
        fs.unlinkSync(tempOut);
      }
    }
  }

  console.log(`\n>>> OVERALL TEST RESULTS: ${totalPassed} passed, ${totalFailed} failed <<<\n`);
  if (totalFailed > 0) {
    process.exit(1);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
