#!/usr/bin/env node

/**
 * OpenClaw Vendor Preparation Script
 *
 * Downloads and prepares OpenClaw tarball for bundling into the app.
 * This allows offline installation without requiring npm/network access.
 */

import { mkdir, writeFile, access } from 'fs/promises';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { createWriteStream } from 'fs';
import { pipeline } from 'stream/promises';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..');
const vendorDir = join(projectRoot, 'src-tauri', 'resources', 'vendor', 'openclaw');
const targetFile = join(vendorDir, 'openclaw.tgz');

// OpenClaw npm package info
const OPENCLAW_PACKAGE = 'openclaw';
const OPENCLAW_VERSION = 'latest'; // or specify a version like '2026.3.2'

async function fileExists(path) {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

async function downloadOpenClawTarball() {
  console.log(`📦 Preparing OpenClaw vendor package...`);

  // Create vendor directory
  await mkdir(vendorDir, { recursive: true });

  // Check if already exists
  if (await fileExists(targetFile)) {
    console.log(`✓ OpenClaw tarball already exists at ${targetFile}`);
    return;
  }

  console.log(`📥 Downloading ${OPENCLAW_PACKAGE}@${OPENCLAW_VERSION}...`);

  try {
    // Get tarball URL from npm registry
    const registryUrl = `https://registry.npmjs.org/${OPENCLAW_PACKAGE}/${OPENCLAW_VERSION}`;
    const response = await fetch(registryUrl);

    if (!response.ok) {
      throw new Error(`Failed to fetch package info: ${response.statusText}`);
    }

    const packageInfo = await response.json();
    const tarballUrl = packageInfo.dist.tarball;

    console.log(`📥 Downloading from ${tarballUrl}...`);

    // Download tarball
    const tarballResponse = await fetch(tarballUrl);
    if (!tarballResponse.ok) {
      throw new Error(`Failed to download tarball: ${tarballResponse.statusText}`);
    }

    // Save to file
    const fileStream = createWriteStream(targetFile);
    await pipeline(tarballResponse.body, fileStream);

    console.log(`✓ OpenClaw tarball saved to ${targetFile}`);
    console.log(`✓ OpenClaw vendor preparation completed`);
  } catch (error) {
    console.error(`❌ Failed to download OpenClaw tarball: ${error.message}`);
    console.log(`⚠️  Continuing without bundled OpenClaw (will use runtime detection)`);
    // Don't fail the build, just skip bundling
  }
}

await downloadOpenClawTarball();
