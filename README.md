# archive

> Safe bindings for [libarchive](https://www.libarchive.org/) with minimum overhead

# Usage

`libarchive` needs to be installed via [vcpkg](https://vcpkg.io/) at build time:
```sh
vcpkg install libarchive
```

`libarchive` is statically linked so there is no runtime dependency.

## Example

```rust
use archive::reader::ArchiveReader;

let reader = std::fs::File::open("tests/simple.zip")?;
let mut archive_reader = ArchiveReader::with_reader(reader)?;

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
