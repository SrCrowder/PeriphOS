# Releases and updates

PeriphOS updates itself from inside the app (Settings > Updates). Installers and the update
manifests are published on GitHub Releases by CI.

## Channels

| Channel     | Published by | When | Manifest |
| ----------- | ------------ | ---- | -------- |
| Development | `.github/workflows/dev-build.yml` | Every push to `main` | `releases/download/nightly/latest.json` (rolling `nightly` prerelease) |
| Stable      | `.github/workflows/release.yml` | Pushing a `vX.Y.Z` tag | `releases/latest/download/latest.json` |

Development builds are versioned `X.Y.Z-dev.N` (N is the workflow run number), so each one is
newer than the previous one and older than the stable `X.Y.Z`. A build defaults to the channel
it was published on; users can switch channels in Settings.

The app only installs files signed with the project's updater key. The public key is in
`src-tauri/tauri.conf.json` (`plugins.updater.pubkey`); the private key exists only in the
repository's secrets and in the maintainer's backup.

## One-time setup (maintainer)

1. Generate the key pair with a password, on a trusted machine:

   ```bash
   npx tauri signer generate -w ~/.tauri/periphos-updater.key
   ```

2. Store the private key and its password as repository secrets:

   ```bash
   gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/periphos-updater.key
   gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD   # paste the password when prompted
   ```

3. Put the contents of `~/.tauri/periphos-updater.key.pub` into `plugins.updater.pubkey` in
   `src-tauri/tauri.conf.json` and commit it.

4. Back up the private key and password (for example in a password manager). **If the key is
   lost, installed copies can no longer be updated** and users must reinstall manually. If it
   leaks, anyone could publish updates that installed apps accept: rotate it immediately.

## Publishing a stable release

1. Set the version in `src-tauri/tauri.conf.json`, `package.json` and the workspace
   `Cargo.toml`, and commit.
2. Tag and push: `git tag v0.2.0 && git push origin v0.2.0`.
3. The Release workflow builds the signed installers and creates the GitHub release with
   `latest.json`.
4. Bump `main` to the next version afterwards, so development builds stay newer than the
   release.

## Notes

- Installers are not code-signed for Windows yet, so SmartScreen warns on first install.
  Updates installed from inside the app are verified with the updater signature regardless.
- Local builds (`npm run tauri build`) do not create update artifacts and need no key.
- Planned hardening: enable `requireSignedVersion` once the Tauri CLI records the version in
  update signatures (it does not as of CLI 2.12.6), which prevents downgrade attacks.
- Linux: AppImage builds will update in place once Linux builds are published; distro
  packages (.deb/.rpm) update through the system package manager.
