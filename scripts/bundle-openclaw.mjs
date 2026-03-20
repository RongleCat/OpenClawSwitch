import { spawn } from "node:child_process";
import { cp, lstat, mkdtemp, mkdir, readdir, readFile, readlink, rm, stat, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { c as createTarball, x } from "tar";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const DEFAULT_OPENCLAW_BUNDLE_CONFIG = Object.freeze({
  packageName: "openclaw",
  version: "2026.3.7",
  registry: "https://registry.npmmirror.com",
});

async function copyDirContents(sourceDir, outputDir) {
  const entries = await readdir(sourceDir, { withFileTypes: true });
  await Promise.all(entries.map(async (entry) => {
    const sourcePath = path.join(sourceDir, entry.name);
    const outputPath = path.join(outputDir, entry.name);
    if (entry.isDirectory()) {
      await mkdir(outputPath, { recursive: true });
      await copyDirContents(sourcePath, outputPath);
      return;
    }
    const entryStat = await lstat(sourcePath);
    if (entryStat.isSymbolicLink()) {
      await symlink(await readlink(sourcePath), outputPath);
      return;
    }
    await cp(sourcePath, outputPath, { recursive: false });
  }));
}

function dependencyPath(outputDir, packageName) {
  if (packageName.startsWith("@")) {
    const [scope, name] = packageName.split("/");
    return path.join(outputDir, "node_modules", scope, name);
  }
  return path.join(outputDir, "node_modules", packageName);
}

function npmPackExecutable() {
  return process.platform === "win32" ? "npm.cmd" : "npm";
}

function normalizeNonEmptyString(value) {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

export async function readOpenClawBundleConfig({
  root = ROOT,
  env = process.env,
} = {}) {
  const packageJson = JSON.parse(await readFile(path.join(root, "package.json"), "utf8"));
  const configured = packageJson.openclawBundle ?? {};

  return {
    packageName: normalizeNonEmptyString(env.OPENCLAW_BUNDLE_PACKAGE)
      ?? normalizeNonEmptyString(configured.packageName)
      ?? DEFAULT_OPENCLAW_BUNDLE_CONFIG.packageName,
    version: normalizeNonEmptyString(env.OPENCLAW_BUNDLE_VERSION)
      ?? normalizeNonEmptyString(configured.version)
      ?? DEFAULT_OPENCLAW_BUNDLE_CONFIG.version,
    registry: normalizeNonEmptyString(env.OPENCLAW_BUNDLE_REGISTRY)
      ?? normalizeNonEmptyString(configured.registry)
      ?? DEFAULT_OPENCLAW_BUNDLE_CONFIG.registry,
  };
}

function bundleSpec({ packageName, version }) {
  return version ? `${packageName}@${version}` : packageName;
}

async function readBundleMetadata(outputDir) {
  const metadataPath = path.join(outputDir, "metadata.json");
  const raw = await readFile(metadataPath, "utf8").catch(() => null);
  if (!raw) {
    return null;
  }

  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

async function extractArchivePackageDir(archivePath, rootDir) {
  await x({ cwd: rootDir, file: archivePath, gzip: true });
  const extractEntries = await readdir(rootDir, { withFileTypes: true });
  return extractEntries.find((entry) => entry.isDirectory())
    ? path.join(rootDir, extractEntries.find((entry) => entry.isDirectory()).name)
    : rootDir;
}

async function archiveHasBundledDependencies(archivePath) {
  const tempDir = await mkdtemp(path.join(tmpdir(), "openclaw-archive-check-"));
  try {
    const packageDir = await extractArchivePackageDir(archivePath, tempDir);
    await assertBundledOpenClawDependencies(packageDir);
    return true;
  } catch {
    return false;
  } finally {
    await rm(tempDir, { recursive: true, force: true });
  }
}

function runCommand(command, args, { cwd } = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd,
      shell: process.platform === "win32",
      stdio: ["ignore", "pipe", "pipe"],
      windowsHide: true,
    });

    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });

    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });

    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) {
        resolve({ stdout, stderr });
        return;
      }

      const details = stderr.trim() || stdout.trim() || `Exit code ${code}`;
      reject(new Error(`${command} ${args.join(" ")} failed: ${details}`));
    });
  });
}

async function installBundledOpenClawDependencies(packageDir, config, {
  npmCommand = npmPackExecutable(),
  runInstallCommand = runCommand,
} = {}) {
  const installArgs = ["install", "--omit=dev", "--fund=false", "--audit=false"];
  if (config.registry) {
    installArgs.push(`--registry=${config.registry}`);
  }

  await runInstallCommand(npmCommand, installArgs, { cwd: packageDir });
}

export async function ensureBundledOpenClawArchive({
  archivePath,
  outputDir = path.dirname(archivePath),
  root = ROOT,
  env = process.env,
  npmCommand = npmPackExecutable(),
  runPackCommand = runCommand,
  installDependencies = installBundledOpenClawDependencies,
} = {}) {
  const config = await readOpenClawBundleConfig({ root, env });
  const desiredSpec = bundleSpec(config);
  const forceRefresh = env.OPENCLAW_BUNDLE_REFRESH === "1";
  const archiveExists = (await stat(archivePath).catch(() => null))?.isFile() ?? false;
  const existingMetadata = await readBundleMetadata(outputDir);
  const existingArchiveHasDependencies = archiveExists
    ? await archiveHasBundledDependencies(archivePath)
    : false;
  const archiveMatchesConfig = archiveExists
    && (
      (!existingMetadata && existingArchiveHasDependencies)
      || (
        existingMetadata
        && existingMetadata.spec === desiredSpec
        && existingMetadata.registry === config.registry
      )
    );

  if (!forceRefresh && archiveMatchesConfig) {
    return {
      archivePath,
      ...config,
      spec: desiredSpec,
    };
  }

  const tempDir = await mkdtemp(path.join(tmpdir(), "openclaw-pack-"));
  try {
    const packArgs = ["pack", desiredSpec, "--silent"];
    if (config.registry) {
      packArgs.push(`--registry=${config.registry}`);
    }

    const { stdout } = await runPackCommand(npmCommand, packArgs, { cwd: tempDir });
    const tarballName = stdout
      .split(/\r?\n/u)
      .map((line) => line.trim())
      .filter(Boolean)
      .at(-1);

    if (!tarballName) {
      throw new Error(`npm pack did not report a tarball for ${desiredSpec}.`);
    }

    const packedArchivePath = path.join(tempDir, tarballName);
    const packedArchive = await stat(packedArchivePath).catch(() => null);
    if (!packedArchive?.isFile()) {
      throw new Error(`npm pack did not create the expected archive: ${packedArchivePath}`);
    }

    const extractRoot = path.join(tempDir, "extract");
    await mkdir(extractRoot, { recursive: true });
    const packageDir = await extractArchivePackageDir(packedArchivePath, extractRoot);

    await installDependencies(packageDir, config, { npmCommand });

    await mkdir(path.dirname(archivePath), { recursive: true });
    await createTarball(
      {
        cwd: extractRoot,
        file: archivePath,
        gzip: true,
      },
      [path.relative(extractRoot, packageDir)],
    );

    return {
      archivePath,
      ...config,
      spec: desiredSpec,
    };
  } finally {
    await rm(tempDir, { recursive: true, force: true });
  }
}

export async function assertBundledOpenClawDependencies(outputDir) {
  const nodeModulesDir = path.join(outputDir, "node_modules");
  const nodeModulesStat = await stat(nodeModulesDir).catch(() => null);
  if (!nodeModulesStat?.isDirectory()) {
    throw new Error("Bundled OpenClaw archive must include node_modules.");
  }

  const packageJsonPath = path.join(outputDir, "package.json");
  const packageJson = JSON.parse(await readFile(packageJsonPath, "utf8"));
  const dependencies = Object.keys(packageJson.dependencies ?? {});
  const missingDependencies = [];

  for (const packageName of dependencies) {
    const dependencyStat = await stat(dependencyPath(outputDir, packageName)).catch(() => null);
    if (!dependencyStat?.isDirectory()) {
      missingDependencies.push(packageName);
    }
  }

  if (missingDependencies.length > 0) {
    throw new Error(`Bundled OpenClaw archive is missing packaged dependencies: ${missingDependencies.join(", ")}.`);
  }
}

export async function bundleOpenClawRuntime({
  archivePath = path.join(ROOT, "src-tauri", "resources", "vendor", "openclaw", "openclaw.tgz"),
  outputDir = path.join(ROOT, "src-tauri", "resources", "vendor", "openclaw"),
  verifyDependencies = assertBundledOpenClawDependencies,
  prepareArchive = ensureBundledOpenClawArchive,
} = {}) {
  await mkdir(outputDir, { recursive: true });
  const preparedArchive = await prepareArchive({ archivePath, outputDir });
  const preservedArchive = await readFile(archivePath);
  const existingEntries = await readdir(outputDir, { withFileTypes: true }).catch(() => []);
  await Promise.all(existingEntries.map((entry) => rm(path.join(outputDir, entry.name), { recursive: true, force: true })));
  await writeFile(path.join(outputDir, "openclaw.tgz"), preservedArchive);
  const extractRoot = path.join(outputDir, ".extract-tmp");
  await mkdir(extractRoot, { recursive: true });
  await x({ cwd: extractRoot, file: archivePath, gzip: true });
  const extractEntries = await readdir(extractRoot, { withFileTypes: true });
  const packageDir = extractEntries.find((entry) => entry.isDirectory()) ? path.join(extractRoot, extractEntries.find((entry) => entry.isDirectory()).name) : extractRoot;
  await copyDirContents(packageDir, outputDir);
  await rm(extractRoot, { recursive: true, force: true });
  await verifyDependencies(outputDir);
  const entryPath = path.join(outputDir, "openclaw.mjs");
  const entryStat = await stat(entryPath);
  if (!entryStat.isFile()) {
    throw new Error(`Bundled OpenClaw entry not found: ${entryPath}`);
  }
  const packageJson = JSON.parse(await readFile(path.join(outputDir, "package.json"), "utf8"));
  await writeFile(
    path.join(outputDir, "metadata.json"),
    JSON.stringify({
      registry: preparedArchive?.registry,
      spec: preparedArchive?.spec ?? bundleSpec({
        packageName: packageJson.name ?? DEFAULT_OPENCLAW_BUNDLE_CONFIG.packageName,
        version: packageJson.version,
      }),
      tarball: path.basename(archivePath),
      name: packageJson.name ?? DEFAULT_OPENCLAW_BUNDLE_CONFIG.packageName,
      version: packageJson.version,
    }, null, 2),
  );
  return { outputDir, entryPath };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  bundleOpenClawRuntime().then(({ outputDir }) => {
    console.log(`Bundled OpenClaw runtime into ${outputDir}`);
  });
}
