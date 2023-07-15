# SPX Codegen
SPX archive compile-time code generator

## Example
From `./tests/test.rs`
```rust
use std::{error::Error, io::Cursor};

use spx_codegen::{ext::FileExt, SpxBuilder};

let mut data = Vec::new();
let mut builder = SpxBuilder::new(Cursor::new(&mut data));

// Add every files of current directory
builder.from_dir("./")?;

// Print generated FileMap code
println!("{}", builder.build());

// Print archive data
println!("{:?}", data);
```
