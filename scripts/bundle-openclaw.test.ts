import { mkdtemp, mkdir, readlink, rm, stat, symlink, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import { afterEach, describe, expect, it } from "vitest"
import { c as createTarball } from "tar"
import { bundleOpenClawRuntime } from "./bundle-openclaw.mjs"

const tempDirs: string[] = []

async function makeTempDir(prefix: string) {
  const dir = await mkdtemp(path.join(tmpdir(), prefix))
  tempDirs.push(dir)
  return dir
}

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })))
})

describe("bundleOpenClawRuntime", () => {
  it("keeps packaged node_modules bin links valid after extracting the archive", async () => {
    const fixtureRoot = await makeTempDir("openclaw-bundle-fixture-")
    const archiveRoot = path.join(fixtureRoot, "archive")
    const packageDir = path.join(archiveRoot, "package")
    const nodeModulesDir = path.join(packageDir, "node_modules")
    const packagePinoDir = path.join(nodeModulesDir, "pino")
    const binDir = path.join(nodeModulesDir, ".bin")

    await mkdir(packagePinoDir, { recursive: true })
    await mkdir(binDir, { recursive: true })
    await writeFile(path.join(packageDir, "openclaw.mjs"), "export {}")
    await writeFile(
      path.join(packageDir, "package.json"),
      JSON.stringify({
        name: "openclaw",
        version: "0.0.0-test",
        dependencies: {
          pino: "1.0.0",
        },
      }),
    )
    await writeFile(path.join(packagePinoDir, "bin.js"), "console.log('pino')")
    await symlink("../pino/bin.js", path.join(binDir, "pino"))

    const archivePath = path.join(fixtureRoot, "openclaw.tgz")
    await createTarball(
      {
        cwd: archiveRoot,
        file: archivePath,
        gzip: true,
      },
      ["package"],
    )

    const outputRoot = await makeTempDir("openclaw-bundle-output-")
    await bundleOpenClawRuntime({
      archivePath,
      outputDir: outputRoot,
      prepareArchive: async () => ({
        archivePath,
        registry: "test",
        spec: "openclaw@0.0.0-test",
      }),
      verifyDependencies: async () => {},
    })

    const bundledBinPath = path.join(outputRoot, "node_modules", ".bin", "pino")
    const bundledBinTarget = await readlink(bundledBinPath)

    await expect(stat(bundledBinPath)).resolves.toBeDefined()
    expect(bundledBinTarget).toBe("../pino/bin.js")
    expect(bundledBinTarget).not.toContain(".extract-tmp")
  })
})
