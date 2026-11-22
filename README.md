# Toybox

[![Support me on Ko-fi](https://img.shields.io/badge/Support%20me%20on%20Ko--fi-72a4f2?style=for-the-badge&logo=kofi&logoColor=white)](https://ko-fi.com/F7X2225YCF)

`toybox` is a reusable Rust framework for audio-plugin GUI and host integration.

## Quick Start

1. Run the local preflight:
   - `bash scripts/run_agent_request.sh`
2. Read the project context:
   - `docs/PROJECT.md`
   - `docs/plans/index.md`
3. Run local validation before submitting changes:
   - `bash scripts/ci_local.sh`

## GUI development

GPUI is enabled by default for new plugin editors on macOS and Windows.
Enable `gpui-vst3` when building a VST3 editor. Toybox embeds GPUI in the
host's native child window and leaves the host in charge of the event loop.
DSP-only consumers can set `default-features = false`.

See [the embedded GPUI guide](docs/GPUI.md) for ownership, input, and lifecycle
requirements. Existing Radiant and Patchbay integrations remain optional during
consumer migration.

## Repository Layout

- `src/`: core framework crates and shared runtime code.
- `patchbay-gui/`: declarative GUI toolkit and rendering/runtime layers.
- `examples/`: minimal CLAP/VST3 reference plugins.
- `docs/`: project constraints, architecture notes, and active plans.
- `scripts/`: local guardrail and CI helper scripts.

## Development Notes

- Keep plugin-specific behavior out of `toybox`; framework behavior should be generic and reusable.
- Use Toybox host adapters and the re-exported `toybox::gpui` types for new editors; keep native host integration inside Toybox.
- For project documentation and planning, see `docs/README.md`.
