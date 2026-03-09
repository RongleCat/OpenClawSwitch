#!/usr/bin/env node

import { access, mkdir, writeFile } from 'node:fs/promises'
import { dirname, join } from 'node:path'
import { platform } from 'node:os'
import { inflateRawSync } from 'node:zlib'
import { fileURLToPath, pathToFileURL } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const projectRoot = join(__dirname, '..')
const nssmDir = join(projectRoot, 'src-tauri', 'resources', 'windows', 'nssm')
const nssmExe = join(nssmDir, 'nssm.exe')
const gitDir = join(projectRoot, 'src-tauri', 'resources', 'windows', 'git')
const mingitZip = join(gitDir, 'mingit.zip')
const gitMetadataPath = join(gitDir, 'metadata.json')

const NSSM_VERSION = '2.24'
const NSSM_DOWNLOAD_URL = `https://nssm.cc/release/nssm-${NSSM_VERSION}.zip`
const GIT_FOR_WINDOWS_LATEST_URL = 'https://github.com/git-for-windows/git/releases/latest'

export function buildGitMetadata(downloadUrl) {
  const assetName = downloadUrl.split('/').pop() || 'mingit.zip'
  const releaseMatch = downloadUrl.match(/\/download\/([^/]+)\//)
  const release = releaseMatch?.[1] ?? null
  const sourceUrl = release
    ? `https://repo.huaweicloud.com/git-for-windows/${release}/${assetName}`
    : downloadUrl

  return {
    release,
    arch: '64-bit',
    archive: 'mingit.zip',
    assetName,
    sourceUrl,
    officialUrl: downloadUrl,
  }
}

export async function fileExists(path) {
  try {
    await access(path)
    return true
  } catch {
    return false
  }
}

export function shouldPrepareWindowsTools({
  buildTarget = process.env.TAURI_TARGET_TRIPLE || '',
  hostPlatform = platform(),
} = {}) {
  if (buildTarget) {
    return buildTarget.includes('windows')
  }
  return hostPlatform === 'win32'
}

export function listZipEntries(zipBuffer) {
  const endSignature = 0x06054b50
  for (let offset = zipBuffer.length - 22; offset >= 0; offset -= 1) {
    if (zipBuffer.readUInt32LE(offset) !== endSignature) continue

    const centralDirectoryOffset = zipBuffer.readUInt32LE(offset + 16)
    const entryCount = zipBuffer.readUInt16LE(offset + 10)
    const entries = []
    let cursor = centralDirectoryOffset

    for (let index = 0; index < entryCount; index += 1) {
      if (zipBuffer.readUInt32LE(cursor) !== 0x02014b50) {
        throw new Error('Invalid zip central directory header')
      }

      const compressionMethod = zipBuffer.readUInt16LE(cursor + 10)
      const compressedSize = zipBuffer.readUInt32LE(cursor + 20)
      const uncompressedSize = zipBuffer.readUInt32LE(cursor + 24)
      const fileNameLength = zipBuffer.readUInt16LE(cursor + 28)
      const extraLength = zipBuffer.readUInt16LE(cursor + 30)
      const commentLength = zipBuffer.readUInt16LE(cursor + 32)
      const localHeaderOffset = zipBuffer.readUInt32LE(cursor + 42)
      const fileNameStart = cursor + 46
      const name = zipBuffer.subarray(fileNameStart, fileNameStart + fileNameLength).toString('utf8')

      entries.push({
        name,
        compressionMethod,
        compressedSize,
        uncompressedSize,
        localHeaderOffset,
      })

      cursor = fileNameStart + fileNameLength + extraLength + commentLength
    }

    return entries
  }

  throw new Error('Invalid zip file: end of central directory not found')
}

export function pickNssmExecutableEntry(entryNames) {
  const normalized = entryNames.map(name => name.replaceAll('\\', '/'))
  return (
    normalized.find(name => name.endsWith('/win64/nssm.exe')) ||
    normalized.find(name => name.endsWith('/win32/nssm.exe')) ||
    normalized.find(name => name.endsWith('/nssm.exe')) ||
    null
  )
}

export function pickLatestWindowsGitReleaseTag(html) {
  const match = html.match(/\/git-for-windows\/git\/releases\/tag\/(v[^"'\s<]+)/)
  return match?.[1] ?? null
}

export function pickLatestMinGitAssetName(html) {
  const match = html.match(/MinGit-[0-9.]+-64-bit\.zip/)
  return match?.[0] ?? null
}

export async function resolveLatestMinGitDownloadUrl({
  fetchImpl = fetch,
  latestUrl = GIT_FOR_WINDOWS_LATEST_URL,
} = {}) {
  const response = await fetchImpl(latestUrl)
  if (!response.ok) {
    throw new Error(`Failed to fetch Git for Windows latest release: ${response.statusText}`)
  }

  const html = await response.text()
  const tag = pickLatestWindowsGitReleaseTag(html)
  const assetName = pickLatestMinGitAssetName(html)
  if (!tag || !assetName) {
    throw new Error('Failed to resolve latest MinGit asset from release page')
  }

  return `https://github.com/git-for-windows/git/releases/download/${tag}/${assetName}`
}

export function extractZipEntry(zipBuffer, entryName) {
  const normalizedName = entryName.replaceAll('\\', '/')
  const entry = listZipEntries(zipBuffer).find(item => item.name.replaceAll('\\', '/') === normalizedName)
  if (!entry) {
    throw new Error(`Zip entry not found: ${entryName}`)
  }

  const localOffset = entry.localHeaderOffset
  if (zipBuffer.readUInt32LE(localOffset) !== 0x04034b50) {
    throw new Error('Invalid zip local header')
  }

  const fileNameLength = zipBuffer.readUInt16LE(localOffset + 26)
  const extraLength = zipBuffer.readUInt16LE(localOffset + 28)
  const dataStart = localOffset + 30 + fileNameLength + extraLength
  const dataEnd = dataStart + entry.compressedSize
  const compressed = zipBuffer.subarray(dataStart, dataEnd)

  if (entry.compressionMethod === 0) {
    return Buffer.from(compressed)
  }
  if (entry.compressionMethod === 8) {
    return inflateRawSync(compressed)
  }

  throw new Error(`Unsupported zip compression method: ${entry.compressionMethod}`)
}

export async function downloadWindowsTools({
  buildTarget = process.env.TAURI_TARGET_TRIPLE || '',
  hostPlatform = platform(),
  fetchImpl = fetch,
  outputDir = nssmDir,
  outputExePath = nssmExe,
  downloadUrl = NSSM_DOWNLOAD_URL,
} = {}) {
  if (!shouldPrepareWindowsTools({ buildTarget, hostPlatform })) {
    console.log('Skipping Windows tools (not a Windows build)')
    return { skipped: true, bundled: false }
  }

  console.log('Preparing Windows tools...')
  await mkdir(outputDir, { recursive: true })

  let nssmResult = { bundled: false }
  if (await fileExists(outputExePath)) {
    console.log(`nssm.exe already exists at ${outputExePath}`)
    nssmResult = { bundled: true, outputExePath }
  } else {
    try {
      console.log(`Downloading NSSM ${NSSM_VERSION}...`)
      const response = await fetchImpl(downloadUrl)
      if (!response.ok) {
        throw new Error(`Failed to download NSSM: ${response.statusText}`)
      }

      const zipBuffer = Buffer.from(await response.arrayBuffer())
      const entryName = pickNssmExecutableEntry(listZipEntries(zipBuffer).map(entry => entry.name))
      if (!entryName) {
        throw new Error('nssm.exe not found in downloaded zip')
      }

      const exeBuffer = extractZipEntry(zipBuffer, entryName)
      await writeFile(outputExePath, exeBuffer)

      console.log(`Bundled NSSM extracted to ${outputExePath}`)
      nssmResult = { bundled: true, outputExePath }
    } catch (error) {
      console.error(`Failed to prepare NSSM: ${error.message}`)
      nssmResult = { bundled: false, error: error.message }
    }
  }

  let gitResult = { bundled: false }
  await mkdir(gitDir, { recursive: true })
  if (await fileExists(mingitZip)) {
    console.log(`MinGit archive already exists at ${mingitZip}`)
    gitResult = { bundled: true, outputZipPath: mingitZip }
  } else {
    try {
      console.log('Resolving latest MinGit release...')
      const gitDownloadUrl = await resolveLatestMinGitDownloadUrl({ fetchImpl })
      const response = await fetchImpl(gitDownloadUrl)
      if (!response.ok) {
        throw new Error(`Failed to download MinGit: ${response.statusText}`)
      }

      await writeFile(mingitZip, Buffer.from(await response.arrayBuffer()))
      console.log(`Bundled MinGit archive saved to ${mingitZip}`)
      gitResult = { bundled: true, outputZipPath: mingitZip, downloadUrl: gitDownloadUrl }
    } catch (error) {
      console.error(`Failed to prepare MinGit: ${error.message}`)
      gitResult = { bundled: false, error: error.message }
    }
  }

  if (gitResult.bundled) {
    try {
      const gitDownloadUrl = gitResult.downloadUrl || await resolveLatestMinGitDownloadUrl({ fetchImpl })
      await writeFile(gitMetadataPath, `${JSON.stringify(buildGitMetadata(gitDownloadUrl), null, 2)}\n`)
      console.log(`MinGit metadata saved to ${gitMetadataPath}`)
    } catch (error) {
      console.error(`Failed to write MinGit metadata: ${error.message}`)
    }
  }

  if (!nssmResult.bundled && !gitResult.bundled) {
    console.log('Continuing without bundled Windows tools (will use runtime download/system PATH)')
  }

  return {
    skipped: false,
    bundled: nssmResult.bundled || gitResult.bundled,
    nssm: nssmResult,
    git: gitResult,
  }
}

const isDirectRun = process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href

if (isDirectRun) {
  await downloadWindowsTools()
}
