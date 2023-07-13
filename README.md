# SPX
A file archive library for bundling and protecting resources using compile-time code generation.

A generated SPX archive file does not contains any metadata.
Each files are encrypted with `AES-128` using md5 hashsum of file path as key.
Offset and size for each files are mapped with perfect hash table and stored inside **compiled binary** without original filename.

## License
Apache-2.0