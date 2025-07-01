# byyte

Byte reading and writing library for Rust.

[![Crates.io](https://img.shields.io/crates/v/byyte.svg)](https://crates.io/crates/byyte)
[![Docs.rs](https://docs.rs/byyte/badge.svg)](https://docs.rs/byyte)

## Example Usage

```rust
// Import the `ByteReader` trait from the `le` module for reading bytes with Little Endian.
use byyte::le::ByteReader;

...
let mut cursor = std::io::Cursor::new(data);
cursor.read_u32()?; // Reads a single u32 from the cursor in Little Endian format.
...

```