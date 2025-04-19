# resonite-core

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