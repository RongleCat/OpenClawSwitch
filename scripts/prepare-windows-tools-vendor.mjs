#!/usr/bin/env node

/**
 * Windows Tools Vendor Preparation Script
 *
 * Downloads Windows-specific tools (nssm.exe) for bundling.
 * Only runs when building for Windows target.
 */

import { mkdir, access } from 'fs/promises';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { createWriteStream } from 'fs';
import { pipeline } from 'stream/promises';
import { platform } from 'os';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');
const windowsToolsDir = join(projectRoot, 'src-tauri', 'resources', 'windows', 'nssm');
const nssmExe = join(windowsToolsDir, 'nssm.exe');

// NSSM download info
const NSSM_VERSION = '2.24';
const NSSM_DOWNLOAD_URL = `https://nssm.cc/release/nssm-${NSSM_VERSION}.zip`;

async function fileExists(path) {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

async function downloadWindowsTools() {
  // Check if we're building for Windows
  const buildTarget = process.env.TAURI_TARGET_TRIPLE || '';
  const isWindowsBuild = buildTarget.includes('windows') || platform() === 'win32';

  if (!isWindowsBuild && !buildTarget) {
    console.log('⏭️  Skipping Windows tools (not a Windows build)');
    return;
  }

  console.log(`📦 Preparing Windows tools...`);

  // Create directory
  await mkdir(windowsToolsDir, { recursive: true });

  // Check if already exists
  if (await fileExists(nssmExe)) {
    console.log(`✓ nssm.exe already exists at ${nssmExe}`);
    return;
  }

  console.log(`📥 Downloading NSSM ${NSSM_VERSION}...`);

  try {
    // Download NSSM zip
    const response = await fetch(NSSM_DOWNLOAD_URL);
    if (!response.ok) {
      throw new Error(`Failed to download NSSM: ${response.statusText}`);
    }

    // For now, we'll need to extract the zip
    // This is a simplified version - in production you'd want to use a zip library
    console.log(`⚠️  NSSM download requires manual extraction`);
    console.log(`   Please download from: ${NSSM_DOWNLOAD_URL}`);
    console.log(`   Extract win64/nssm.exe to: ${nssmExe}`);
    console.log(`⚠️  Continuing without bundled NSSM (will use system PATH)`);
  } catch (error) {
    console.error(`❌ Failed to prepare Windows tools: ${error.message}`);
    console.log(`⚠️  Continuing without bundled tools (will use system PATH)`);
  }
}

await downloadWindowsTools();
