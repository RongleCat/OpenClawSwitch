import { readFileSync } from "node:fs"
import { resolve } from "node:path"
import { describe, expect, it } from "vitest"

const readJson = (relativePath: string) =>
  JSON.parse(readFileSync(resolve(process.cwd(), relativePath), "utf8")) as {
    build: {
      beforeDevCommand: string
      beforeBuildCommand: string
    }
    tauri: {
      bundle: {
        resources?: string[]
      }
    }
  }

const readText = (relativePath: string) => readFileSync(resolve(process.cwd(), relativePath), "utf8")

describe("tauri build commands", () => {
  it("keeps the base config cross-platform safe", () => {
    const tauriConfig = readJson("src-tauri/tauri.conf.json")

    expect(tauriConfig.build.beforeDevCommand).not.toContain("npm run")
    expect(tauriConfig.build.beforeBuildCommand).not.toContain("npm run")
    expect(tauriConfig.build.beforeBuildCommand).not.toContain("prepare-openclaw-vendor")
    expect(tauriConfig.build.beforeBuildCommand).not.toContain("prepare-windows-tools-vendor")
    expect(tauriConfig.tauri.bundle.resources ?? []).not.toContain("resources/windows/**")
  })

  it("keeps Windows-only assets in a Windows override config", () => {
    const windowsConfig = readJson("src-tauri/tauri.windows.conf.json")

    expect(windowsConfig.build.beforeBuildCommand).toContain("prepare-windows-tools-vendor")
    expect(windowsConfig.tauri.bundle.resources).toContain("resources/windows/**")
  })

  it("routes CI builds through the right platform-specific commands", () => {
    const workflow = readText(".github/workflows/release.yml")

    expect(workflow).toContain("npm run tauri:build:mac")
    expect(workflow).toContain("npm run tauri:build:windows")
  })
})
