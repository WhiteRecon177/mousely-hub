# Releasing updates

Mousely Hub updates itself from GitHub Releases (Tauri updater).

## One-time setup

1. Generate the signing keys (keep the private one secret, never commit it):
   `npm run tauri signer generate -- -w %USERPROFILE%\.tauri\mousely.key`
2. Paste the **public** key (the `.key.pub` content) in `src-tauri/tauri.conf.json` -> `plugins.updater.pubkey`.
3. In the same file replace `WhiteRecon177` in `endpoints` with your GitHub user/org name.
4. On GitHub: repo -> Settings -> Secrets and variables -> Actions, add
   - `TAURI_SIGNING_PRIVATE_KEY` = content of the private `.key` file
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` = the password you chose (leave empty if none)

## Every new version

1. Raise the version in three places (same number): `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`.
2. Commit and push, then create and push a tag:
   `git tag v0.1.1` and `git push origin v0.1.1`
3. The GitHub Action builds, signs and publishes the Release with `latest.json`.
   Installed apps find it on their next check (a few seconds after start, then every 6 hours, or "Settings -> Updates -> Check").

Local builds (`npm run tauri build`) do not need the key: updater files are only created by the Action.
