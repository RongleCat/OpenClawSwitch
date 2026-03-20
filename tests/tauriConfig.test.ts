import { readFileSync } from "node:fs"
import { resolve } from "node:path"
import { describe, expect, it } from "vitest"

const readJson = (relativePath: string) =>
  JSON.parse(readFileSync(resolve(process.cwd(), relativePath), "utf8")) as {
    build: {
      beforeDevCommand: string
      beforeBuildCommand: string
    }
    bundle?: {
      resources?: string[]
    }
    tauri?: {
      bundle: {
        resources?: string[]
      }
    }
    app?: {
      windows?: Array<{
        width?: number
        height?: number
        minWidth?: number
        minHeight?: number
      }>
    }
  }

const readText = (relativePath: string) => readFileSync(resolve(process.cwd(), relativePath), "utf8")

describe("tauri build commands", () => {
  it("keeps the base config aligned with bundled runtime resources", () => {
    const tauriConfig = readJson("src-tauri/tauri.conf.json")
    const bundleResources = tauriConfig.bundle?.resources ?? tauriConfig.tauri?.bundle?.resources ?? []

    expect(tauriConfig.build.beforeDevCommand).not.toContain("npm run")
    expect(tauriConfig.build.beforeDevCommand).toContain("bundle-openclaw")
    expect(tauriConfig.build.beforeDevCommand).toContain("bundle-node-runtime")
    expect(tauriConfig.build.beforeBuildCommand).not.toContain("npm run")
    expect(tauriConfig.build.beforeBuildCommand).toContain("bundle-openclaw")
    expect(tauriConfig.build.beforeBuildCommand).toContain("bundle-node-runtime")
    expect(bundleResources).toContain("resources/vendor/openclaw")
    expect(bundleResources).toContain("resources/vendor/node")
    expect(bundleResources).not.toContain("resources/windows/**")
  })

  it("keeps the windows override limited to platform packaging tweaks", () => {
    const windowsConfig = readJson("src-tauri/tauri.windows.conf.json")
    const bundleResources = windowsConfig.bundle?.resources ?? windowsConfig.tauri?.bundle?.resources ?? []

    expect(windowsConfig.build.beforeBuildCommand).not.toContain("prepare-windows-tools-vendor")
    expect(bundleResources).not.toContain("resources/windows/**")
  })

  it("routes CI builds through the right platform-specific commands", () => {
    const workflow = readText(".github/workflows/release.yml")

    expect(workflow).toContain("npm run tauri:build:mac")
    expect(workflow).toContain("npm run tauri:build:windows")
  })

  it("sets the compact shell window size in tauri config", () => {
    const tauriConfig = readJson("src-tauri/tauri.conf.json")
    const window = tauriConfig.app?.windows?.[0]

    expect(window?.width).toBe(1000)
    expect(window?.height).toBe(600)
    expect(window?.minWidth).toBe(960)
    expect(window?.minHeight).toBe(580)
  })
})
