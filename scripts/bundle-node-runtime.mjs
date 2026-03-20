import { access, cp, mkdir, readdir, rm } from "node:fs/promises";
import { constants } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

const TARGET_SPECS = {
  "win32-x64": { executable: "node.exe" },
  "win32-arm64": { executable: "node.exe" },
  "darwin-x64": { executable: path.join("bin", "node") },
  "darwin-arm64": { executable: path.join("bin", "node") }
};

export function getNodeTargetSpec(targetId) {
  const spec = TARGET_SPECS[targetId];
  if (!spec) {
    throw new Error(`Unsupported Node runtime target: ${targetId}`);
  }
  return spec;
}

function getCurrentTargetId() {
  if (process.platform === "win32" && process.arch === "x64") return "win32-x64";
  if (process.platform === "win32" && process.arch === "arm64") return "win32-arm64";
  if (process.platform === "darwin" && process.arch === "x64") return "darwin-x64";
  if (process.platform === "darwin" && process.arch === "arm64") return "darwin-arm64";
  return null;
}

export async function bundleNodeRuntime({
  sourceRoot = path.join(ROOT, "src-tauri", "resources", "vendor", "node-source"),
  outputRoot = path.join(ROOT, "src-tauri", "resources", "vendor", "node"),
  targets
} = {}) {
  await mkdir(outputRoot, { recursive: true });
  const existingEntries = await readdir(outputRoot, { withFileTypes: true }).catch(() => []);
  await Promise.all(existingEntries.map((entry) => rm(path.join(outputRoot, entry.name), { recursive: true, force: true })));

  const explicitTargets = targets ?? [];
  const defaultTarget = getCurrentTargetId();
  const targetList = explicitTargets.length > 0
    ? explicitTargets
    : defaultTarget
      ? [defaultTarget]
      : Object.keys(TARGET_SPECS);

  const bundledTargets = [];
  for (const targetId of targetList) {
    const spec = getNodeTargetSpec(targetId);
    const sourceDir = path.join(sourceRoot, targetId);
    const executablePath = path.join(sourceDir, spec.executable);
    const targetOutputDir = path.join(outputRoot, targetId);
    await mkdir(targetOutputDir, { recursive: true });

    const hasPrebuiltRuntime = await access(executablePath, constants.F_OK)
      .then(() => true)
      .catch(() => false);

    if (hasPrebuiltRuntime) {
      await cp(sourceDir, targetOutputDir, { recursive: true });
    } else if (targetId === defaultTarget) {
      const fallbackExecutable = path.join(targetOutputDir, spec.executable);
      await mkdir(path.dirname(fallbackExecutable), { recursive: true });
      await cp(process.execPath, fallbackExecutable);
    } else {
      throw new Error(`Missing Node runtime payload for target ${targetId}: ${executablePath}`);
    }

    bundledTargets.push({
      targetId,
      outputDir: targetOutputDir,
      executablePath: path.join(targetOutputDir, spec.executable)
    });
  }

  return bundledTargets;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  bundleNodeRuntime().then((targets) => {
    console.log(`Bundled Node runtimes: ${targets.map((target) => target.targetId).join(", ")}`);
  });
}
