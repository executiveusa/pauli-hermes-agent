# NPM Package Management

## Updating pi-skills

`templates/pi-skills/` contains regular tracked files (not a submodule). To update from the upstream [badlogic/pi-skills](https://github.com/badlogic/pi-skills) repo:

```bash
# Remove the old files
rm -rf templates/pi-skills

# Download and extract the latest tarball
curl -sL https://github.com/badlogic/pi-skills/archive/refs/heads/main.tar.gz \
  | tar xz --strip-components=1 -C $(mkdir -p templates/pi-skills && echo templates/pi-skills)

# Stage and commit
git add templates/pi-skills/
git commit -m "update pi-skills to latest"
```

## Release Workflow

### Version Commands

- **`npm version prepatch --preid=beta`** — `1.2.4` → `1.2.5-beta.0`
  Bumps patch and starts a prerelease. Use once to begin a patch beta cycle.

- **`npm version preminor --preid=beta`** — `1.2.4` → `1.3.0-beta.0`
  Bumps minor and starts a prerelease. Use once to begin a minor beta cycle.

- **`npm version premajor --preid=beta`** — `1.2.4` → `2.0.0-beta.0`
  Bumps major and starts a prerelease. Use once to begin a major beta cycle.

- **`npm version prerelease --preid=beta`** — increments the prerelease number only:
  `1.2.5-beta.0` → `1.2.5-beta.1` → `1.2.5-beta.2` → ...

- **Finalize a release** (strip the prerelease tag):
  - `npm version patch` — `1.2.5-beta.2` → `1.2.5`
  - `npm version minor` — `1.3.0-beta.2` → `1.3.0`
  - `npm version major` — `2.0.0-beta.2` → `2.0.0`

### Typical Workflow

```
1. npm version prepatch --preid=beta     # start beta cycle (e.g. 1.2.5-beta.0)
2. npm publish --tag beta                # publish beta
3. npm version prerelease --preid=beta   # bump beta number (1.2.5-beta.1)
4. npm publish --tag beta                # publish next beta
   ... repeat 3-4 as needed ...
5. npm version patch                     # finalize (1.2.5)
6. npm publish                           # publish stable to latest tag
```

### Publishing

```bash
# Beta release (does not affect `latest` tag — safe for testing)
npm publish --tag beta

# Stable release (updates `latest` — what users get with `npm install thepopebot`)
npm publish
```

### Verifying a Release

```bash
# Dry run — see what files would be in the tarball
npm pack --dry-run

# Check a published version
npm view thepopebot versions --json
npm pack thepopebot@<version> && tar tzf thepopebot-<version>.tgz
```
