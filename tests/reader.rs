use archive::reader::ArchiveReader;
use std::io::{read_to_string, Cursor, Read, Seek};

static SIMPLE_ZIP: &[u8] = include_bytes!("simple.zip");
static NESTED_ZIP: &[u8] = include_bytes!("nested.zip");

fn test_simple_zip_content(_simple_zip_content: impl Read + Seek + Send, skip_read_first: bool) {
    let mut archive_reader = ArchiveReader::open_io(Cursor::new(SIMPLE_ZIP)).unwrap();

    let first_entry = archive_reader.next_entry().unwrap().unwrap();
    assert_eq!(first_entry.pathname_utf8().unwrap(), "a");
    assert_eq!(first_entry.pathname_mb().unwrap().to_bytes(), b"a");
    if !skip_read_first {
        assert_eq!(read_to_string(first_entry.into_reader()).unwrap(), "foo");
    }

    let second_entry = archive_reader.next_entry().unwrap().unwrap();
    assert_eq!(second_entry.pathname_utf8().unwrap(), "b");
    assert_eq!(read_to_string(second_entry.into_reader()).unwrap(), "bar");

    assert!(archive_reader.next_entry().unwrap().is_none());
}

#[test]
fn test_simple() {
    test_simple_zip_content(Cursor::new(SIMPLE_ZIP), false)
}

#[test]
fn test_skip_entry() {
    test_simple_zip_content(Cursor::new(SIMPLE_ZIP), true)
}

#[test]
fn test_nested() {
    let mut archive_reader = ArchiveReader::open_io(Cursor::new(NESTED_ZIP)).unwrap();
    let simple_zip_entry = archive_reader.next_entry().unwrap().unwrap();
    assert_eq!(simple_zip_entry.pathname_utf8().unwrap(), "simple.zip");
    test_simple_zip_content(simple_zip_entry.into_reader(), false);
}
