# libDS: ds-rs C Bindings

This library provides C compatible binding to the `ds-rs` FRC Driver Station library

`./libDS.h` provides the latest header describing the C API exported by this library. The library can be built into both dynamic and static libraries with `cargo build`. These libraries are fully compatible with C and can be used to control FRC robots in other environments where a pure Rust applicaion is not practical.



## Limitations

This library does not export the full surface area of `ds-rs` in its current state. Notably, there is no way for C API consumers to provide joystick values to the driver station. This may be changed in the future.



## Building

The Rust library can be compiled using `cargo build`, optionally providing the `--release` flag to optimize the produced libraries.

The header is generated with [cbindgen](https://github.com/eqrion/cbindgen/), if certain changes to the library are made that would require a regeneration of the header (e.g. adding exports to the crate), the header can be regenerated with the command `cbindgen --config cbindgen.toml --crate libDS --output libDS.h` after following the [installation instructions](https://github.com/eqrion/cbindgen/#quick-start) for the cbindgen CLI tool.