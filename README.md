# resonite-core

<picture>
    <source srcset="https://github.com/vlams1/resonite-core/raw/master/assets/logo-dark.svg" media="(prefers-color-scheme: dark)">
    <source srcset="https://github.com/vlams1/resonite-core/raw/master/assets/logo-light.svg" media="(prefers-color-scheme: light)">
    <img src="" align="right" width="300" height="300">
</picture>

[![Crates.io](https://img.shields.io/crates/v/resonite-core.svg)](https://crates.io/crates/resonite-core)
[![Docs.rs](https://docs.rs/resonite-core/badge.svg)](https://docs.rs/resonite-core)\
[![MIT](https://img.shields.io/badge/license-MIT-43B.svg)](https://github.com/vlams1/resonite-core/blob/master/LICENSE-MIT)
[![APACHE 2.0](https://img.shields.io/badge/license-Apache-43B.svg)](https://github.com/vlams1/resonite-core/blob/master/LICENSE-APACHE)

**resonite-core** is a pure Rust implementation of file formats used by the game [Resonite](https://resonite.com).\
It provides parsing and serialization support for various Resonite-specific file types.

*This library is not affiliated with the creators of Resonite.*

### Currently Supported Formats
- `AnimJ` – JSON animation data
- `AnimX` – Binary animation data

### Example

Converting AnimJ to AnimX

```rust
use resonite_core::animation::Animation;

let anim: Animation = serde_json::from_str(/* AnimJ */)?;
let mut buf = Vec::new();
anim.write_animx(&mut buf);
```