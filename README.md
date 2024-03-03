# Blind Image Steganograph

https://github.com/tiquthon/blind-image-steganograph

By Thimo "Tiquthon" Neumann 2024

With "Blind Image Steganograph" you can hide text or arbitrary files within an image without changing its perceptual quality significantly using Blind Image Steganography which stores the data in the [Least Significant Bits (LSB)](https://en.wikipedia.org/wiki/Bit_numbering#Least_significant_bit) and thus does not need the original image when extracting the data.

You are free to copy, modify, and distribute "Blind Image Steganograph" with attribution under the terms of the MIT license.
See the [LICENSE file](./LICENSE) for details.

# Develop The Project

Before developing "Blind Image Steganograph" you need:
- [Rust](https://www.rust-lang.org/) Toolchain with clippy and cargo fmt
- *for checking dependencies:*
  - *either* [Cargo Audit](https://crates.io/crates/cargo-audit), install with `cargo install cargo-audit --locked`
  - *or* [Cargo Deny](https://crates.io/crates/cargo-deny), install with `cargo install cargo-deny --locked`

## Git Hook

There is a Git Hook inside [githooks/](./githooks/) helping you providing good code quality.
Install it inside [.git/hooks/.](./.git/hooks/) and make it executable.

---

README created with the help of [https://github.com/ddbeck/readme-checklist/checklist.md](https://github.com/ddbeck/readme-checklist/blob/a234904e6d0030fe4f56f26fa1ba6e4d300b39ba/checklist.md).
