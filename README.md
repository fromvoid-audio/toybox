# Toybox

<a href='https://ko-fi.com/F7X2225YCF' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://storage.ko-fi.com/cdn/kofi6.png?v=6' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>

Toybox is a Rust framework for audio plug-ins. It provides shared host adapters and embeds native GPUI editors in DAWs on macOS and Windows.

GPUI is enabled by default. Enable `gpui-vst3` for VST3 editors or set `default-features = false` for DSP-only use. The host owns its event loop and native window lifecycle.

Run `bash scripts/run_agent_request.sh` before editing and `bash scripts/ci_local.sh` to validate changes locally.

[Embedded GPUI guide](docs/GPUI.md) · [Project docs](docs/README.md) · [MIT license](LICENSE)

Bundled fonts retain their own licenses. See [Ioskeley Mono](assets/IoskeleyMono/OFL.txt) and [Sometype Mono](assets/Sometype_Mono/OFL.txt).
