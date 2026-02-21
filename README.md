# Kira (Rust version)

Kira is an example compiler for PKU compiler course, it can compile SysY language into [Koopa IR](https://github.com/pku-minic/koopa) and RISC-V assembly.

`kira-rs` is written in Rust, for C++ example, please see [`kira-cpp`](https://github.com/pku-minic/kira-cpp).

## Usage

```sh
# compiler `input.c` to Koopa IR
cargo run -- -koopa input.c -o output.koopa
# compiler `input.c` to RISC-V assembly
cargo run -- -riscv input.c -o output.S
```

## Examples

the `examples/c` directory contains a list of example programs that's provided by [kecc-public](https://github.com/kaist-cp/kecc-public/). 90% of them are supported except some programs that contains `sizeof`, `struct` .etc. and `examples/koopa` directory contains the koopa ir that is generated via this project.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## Acknowledgements

- [kecc-public](https://github.com/kaist-cp/kecc-public/). Another great educational compiler. We used their unittest suite.

## Copyright and License

Copyright (C) 2022-2025 MaxXing. License GPLv3.
