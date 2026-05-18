# Workflow: Install a library CLI

## Goal
Install or discover prebuilt Printing Press CLIs before generating a new one.

## Steps
1. List available CLIs:
   ```bash
   pp-list
   ```
2. Search by target domain/category:
   ```bash
   pp-search <keyword>
   ```
3. Install exact CLI package/bundle:
   ```bash
   pp-install <slug-or-bundle>
   ```
4. Validate the CLI exists and supports compact structured output:
   ```bash
   <slug>-pp-cli --help
   <slug>-pp-cli <command> --json --compact
   ```
5. Add a registry entry using `templates/cli-registry-entry.yaml`.

## Exit criteria
- CLI installed and executable.
- At least one read-only command runs with `--json --compact`.
- Registry status set to `installed`.
