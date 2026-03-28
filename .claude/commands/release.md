# release

Build and install a new release of Valens on this machine.

## Usage

`/release <bump-type>`

**bump-type** must be one of: `patch`, `minor`, `major`

**Examples:**
- `/release patch` — bug fixes, small improvements (0.1.0 → 0.1.1)
- `/release minor` — new features (0.1.0 → 0.2.0)
- `/release major` — breaking changes or major milestones (0.1.0 → 1.0.0)

## What this skill does

Given a bump type, this skill will:

1. Read the current version from `Cargo.toml`
2. Compute the new version by incrementing the appropriate component
3. Update the `version` field in `Cargo.toml`
4. Run `./check.sh` to verify the release is clean (format, clippy, tests, WASM build)
5. Commit the version bump with message `Release v{VERSION}`
6. Tag the commit `v{VERSION}`
7. Build the release binary and install `Valens.app` to `/Applications/`

## Rules you must follow

- **Always run check.sh first**: Never tag or install if checks fail.
- **Only update the package version**: Change the `version` line in `[package]`, not any `version` lines in `[dependencies]`.
- **Commit before tagging**: The tag must point to the version-bump commit.
- **Data is safe**: `~/Library/Application Support/valens/` is never touched by the install step.
- **Rollback**: The previous `.app` is backed up to `/Applications/Valens.app.bak` automatically. To roll back: `rm -rf /Applications/Valens.app && mv /Applications/Valens.app.bak /Applications/Valens.app`

## Step-by-step instructions

$ARGUMENTS

1. **Read current version**: Read `Cargo.toml`, extract the `version = "..."` value from the `[package]` section.

2. **Compute new version**: Split into `major.minor.patch`. Increment the component named in the bump type; reset all lower components to 0.
   - `patch`: patch += 1
   - `minor`: minor += 1, patch = 0
   - `major`: major += 1, minor = 0, patch = 0

3. **Update `Cargo.toml`**: Replace the `version = "X.Y.Z"` line in `[package]` with the new version.

4. **Run checks**:
   ```sh
   ./check.sh
   ```
   If any check fails, restore the original version in `Cargo.toml` and report the error. Do not proceed.

5. **Commit the version bump**:
   ```sh
   git add Cargo.toml Cargo.lock
   git commit -m "Release v{VERSION}"
   ```

6. **Tag the release**:
   ```sh
   git tag v{VERSION}
   ```

7. **Build and install**:
   ```sh
   bash scripts/bundle.sh
   ```
   This builds the release binary, creates the `.app` bundle with the correct icon and `Info.plist`, backs up the existing installation, and copies the new bundle to `/Applications/Valens.app`.

8. **Confirm success**: Report the new version, the git tag, and remind the user to quit and relaunch the app if it was running.
