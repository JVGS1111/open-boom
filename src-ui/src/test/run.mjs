import esbuild from "esbuild";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const testFile = path.join(__dirname, "EqualizerStudioDrawer.test.tsx");
const tempOut = path.join(__dirname, ".test-bundle.mjs");

async function main() {
  try {
    await esbuild.build({
      entryPoints: [testFile],
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
    const results = mod.runEqualizerStudioDrawerTests();

    console.log("\n=======================================================");
    console.log(" EqualizerStudioDrawer & Spline Curve Test Suite");
    console.log("=======================================================\n");

    let passed = 0;
    let failed = 0;

    results.forEach((r) => {
      if (r.passed) {
        console.log(`  ✓ ${r.name}`);
        passed++;
      } else {
        console.error(`  ✗ ${r.name}`);
        console.error(`    Error: ${r.error}`);
        failed++;
      }
    });

    console.log("\n-------------------------------------------------------");
    console.log(` Summary: ${passed} passed, ${failed} failed (${results.length} total)`);
    console.log("=======================================================\n");

    if (failed > 0) {
      process.exit(1);
    }
  } finally {
    if (fs.existsSync(tempOut)) {
      fs.unlinkSync(tempOut);
    }
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
