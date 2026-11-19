#!/usr/bin/env node
/**
 * Writes the `latest.json` manifest read by the in-app updater.
 *
 * Usage:
 *   node scripts/updater-manifest.mjs --version 0.2.0 --base-url <download url> \
 *     --nsis path/to/setup.exe [--msi path/to/installer.msi] [--notes "text"] [--out latest.json]
 *
 * Each installer must have its `.sig` file (created by `tauri build` when
 * `bundle.createUpdaterArtifacts` is enabled) next to it.
 */
import { readFileSync, writeFileSync } from "node:fs";
import { basename } from "node:path";
import { parseArgs } from "node:util";

const { values } = parseArgs({
  options: {
    version: { type: "string" },
    "base-url": { type: "string" },
    nsis: { type: "string" },
    msi: { type: "string" },
    notes: { type: "string", default: "" },
    out: { type: "string", default: "latest.json" },
  },
});

for (const required of ["version", "base-url", "nsis"]) {
  if (!values[required]) {
    console.error(`Missing --${required}`);
    process.exit(1);
  }
}

const baseUrl = values["base-url"].replace(/\/$/, "");

function entry(file) {
  return {
    signature: readFileSync(`${file}.sig`, "utf8").trim(),
    url: `${baseUrl}/${encodeURIComponent(basename(file))}`,
  };
}

const nsis = entry(values.nsis);
const platforms = {
  // Generic key used by apps installed with either installer, then the
  // installer-specific keys the updater prefers when present.
  "windows-x86_64": nsis,
  "windows-x86_64-nsis": nsis,
};
if (values.msi) platforms["windows-x86_64-msi"] = entry(values.msi);

const manifest = {
  version: values.version,
  notes: values.notes,
  pub_date: new Date().toISOString(),
  platforms,
};

writeFileSync(values.out, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`Wrote ${values.out} for ${values.version}`);
