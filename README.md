# AT Commands builder and parser for Rust #![no_std] [![crates.io](https://img.shields.io/crates/v/at-commands.svg)](https://crates.io/crates/at-commands) [![Documentation](https://docs.rs/at-commands/badge.svg)](https://docs.rs/at-commands)

This crate can be used to build at command style messages efficiently.

There is experimental support for parsing them too, but that isn't complete and isn't too efficient just yet.
To access this, you'll need to enable the parser feature. Semver does apply to it, though.

(Help and feedback would be appreciated with this feature)

## Usage

Builder:
```rust
use at_commands::builder::CommandBuilder;

let mut buffer = [0; 128];

// Make a query command
let result = CommandBuilder::create_query(&mut buffer, true)
    .named("+MYQUERY")
    .finish()
    .unwrap();

// Buffer now contains "AT+MYQUERY?"
// Copy or DMA the resulting slice to the device.

// Make a set command
let result = CommandBuilder::create_set(&mut buffer, false)
    .named("+MYSET")
    .with_int_parameter(42)
    .finish()
    .unwrap();

// Buffer now contains "+MYSET=42"
// Copy or DMA the resulting slice to the device.
```

Parser:
```rust
let (x, y, z) = CommandParser::parse(b"+SYSGPIOREAD:654,\"true\",-65154\r\nOK\r\n")
    .expect_identifier(b"+SYSGPIOREAD:")
    .expect_int_parameter()
    .expect_string_parameter()
    .expect_int_parameter()
    .expect_identifier(b"\r\nOK\r\n")
    .finish() 
    .unwrap();

// x = 654
// y = "true"
// z = -65154
```


## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.