# Toybox

Toybox is a Rust framework for audio plug-ins. It provides shared host adapters and embeds native GPUI editors in DAWs on macOS and Windows.

- Host-owned event loop and native window lifecycle for embedded GPUI plug-in editors.
- Shared CLAP and VST3 adapters with state validation and DSP utilities.
- Modular Cargo features: GPUI enabled by default, `gpui-vst3` for VST3 editors, or `default-features = false` for DSP-only use.

## Build

Requires Rust. VST3 checks and builds also require the [Steinberg VST3 SDK](https://github.com/steinbergmedia/vst3sdk) (`VST3_SDK_DIR`).

```sh
cargo build --locked                                                         # build default crate
cargo test --locked -p toybox                                                # run tests
VST3_SDK_DIR=/path/to/vst3sdk cargo test --locked -p toybox --features vst3  # VST3 tests
bash scripts/ci_local.sh                                                     # run local CI checks
```

## About

I’m [PORTALSURFER](https://soundcloud.com/portalsurfer). I make music and build instruments and tools for making it under [fromvoid](https://github.com/fromvoid-audio). I use AI extensively in that work and share these projects so more people can make music, whatever their budget.

I build this software for myself first and foremost and offer no guarantees of support. That said, I welcome code, documentation, and interesting ideas, and I’m happy to add them when we align on the direction.

Copyright wasn't just inconvenienced by AI; perhaps it was always the wrong bet.

We treated ideas like property, then acted surprised when progress slowed and wealth pooled upward. But evolution has always followed a simple pattern: copying, mutation, and selection. Ideas, like living things, develop through variation and iteration.

Perhaps the mistake was never copying, but our insistence on preventing it.

Soon, software will be something we simply wish into existence, endlessly adapting to our needs. When creation becomes abundant, artificial scarcity loses its purpose.

fromvoid is built in the open, not as charity, but as a refusal to restrict the evolution of ideas.

Copy. Mutate. Select. Repeat.

## Support

If you’d like to help cover development costs, you can chip in on Ko-fi:

<a href='https://ko-fi.com/F7X2225YCF' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://storage.ko-fi.com/cdn/kofi6.png?v=6' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>

---

[Embedded GPUI guide](docs/GPUI.md) · [Project docs](docs/README.md) · [MIT license](LICENSE)

Bundled fonts retain their own licenses: [Ioskeley Mono](assets/IoskeleyMono/OFL.txt) and [Sometype Mono](assets/Sometype_Mono/OFL.txt).
