# Blind Image Steganography

https://github.com/tiquthon/blind-image-steganograph/lib

By Thimo "Tiquthon" Neumann 2024

With the Rust library "Blind Image Steganography" you can hide text or arbitrary files within an image without changing its perceptual quality significantly using Blind Image Steganography which stores the data in the [Least Significant Bits (LSB)](https://en.wikipedia.org/wiki/Bit_numbering#Least_significant_bit) and thus does not need the original image when extracting the data.

You are free to copy, modify, and distribute "Blind Image Steganograph" with attribution under the terms of the MIT license.
See the [LICENSE file](./LICENSE) for details.

# Use Library In Your Rust Project

Add the dependency to your project within your `Cargo.toml`:
```toml
[dependencies]
blind-image-steganography = { git = "https://github.com/tiquthon/blind-image-steganograph.git" }
```

It's good practice to reference a direct git tag or commit hash like:
```toml
[dependencies]
blind-image-steganography = { git = "https://github.com/tiquthon/blind-image-steganograph.git", tag = "v0.0.0" }
```
or
```toml
[dependencies]
blind-image-steganography = { git = "https://github.com/tiquthon/blind-image-steganograph.git", rev = "abcdefgh" }
```

*More is W.I.P.*

---

README created with the help of [https://github.com/ddbeck/readme-checklist/checklist.md](https://github.com/ddbeck/readme-checklist/blob/a234904e6d0030fe4f56f26fa1ba6e4d300b39ba/checklist.md).
