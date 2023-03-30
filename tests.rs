#[cfg(test)]
mod tests {
    use super::*;

    fn make_small_fs() -> FileSystem<16, 64, 255, 8, 512, 32, 8> {
        FileSystem::new(ramdisk::RamDisk::new())
    }

    #[test]
    fn test_empty() {
        let mut sys = make_small_fs();
        assert!(!sys.directory_exists());
        assert!(sys.inode_for("test") == FileSystemResult::Err(FileSystemError::FileNotFound));
    }

    #[test]
    fn test_short_write() {
        let mut sys = make_small_fs();
        let f1 = sys.open_create("one.txt").unwrap();
        sys.write(f1, "This is a test.".as_bytes()).unwrap();
        let mut buffer = [0; 50];
        sys.close(f1).unwrap();
        let f2 = sys.open_read("one.txt").unwrap();
        let bytes_read = sys.read(f2, &mut buffer).unwrap();
        assert_eq!(bytes_read, 15);
        let s = core::str::from_utf8(&buffer[0..bytes_read]).unwrap();
        assert_eq!(s, "This is a test.");
    }
}