# akv

> Safe bindings for [libarchive](https://www.libarchive.org/) with minimum overhead

# Example

```rust
use akv::reader::ArchiveReader;

let io_reader = std::fs::File::open("tests/simple.zip")?;
let mut archive_reader = ArchiveReader::open_io(io_reader)?;

while let Some(entry) = archive_reader.next_entry()? {
    println!("Entry name: {}", entry.pathname_utf8()?);
    let entry_reader = entry.into_reader();
    println!(
        "Entry content: {}",
        std::io::read_to_string(entry_reader)?
    );
}
std::io::Result::Ok(())
```
