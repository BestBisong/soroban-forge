# Configuration

## Global Flags

| Flag | Env Variable | Default | Description |
|------|-------------|---------|-------------|
| `--verbose` | `SOROBAN_FORGE_VERBOSE` | `false` | Debug logging |
| `--quiet` | `SOROBAN_FORGE_QUIET` | `false` | Suppress output |
| `--json` | `SOROBAN_FORGE_JSON` | `false` | JSON output |
| `--yes`/`-y` | `SOROBAN_FORGE_YES` | `false` | Auto-confirm |
| `--cwd`/`-C` | — | cwd | Run from DIR |
| `--offline` | `SOROBAN_FORGE_OFFLINE` | `false` | No network |

Precedence: CLI flag > env var > forge.toml > default.
