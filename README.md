# SPX
A file archive library for bundling and protecting resources using compile-time code generation.

A generated SPX archive file does not contains any metadata.
Each files are encrypted with `Chacha20` using sha256 hash of file path as key.
Offset and size for each files are mapped with perfect hash table and stored inside **compiled binary** without original filename. But without correct file name, it is impossible to find correct offset size pair and original file data.

## License
Apache-2.0