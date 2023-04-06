#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK_SIZE: usize = 64;
    const MAX_FILES_STORED: usize = 32;

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
        
    const LONG_DATA: &str = "This is a much, much longer message.
    It crosses a number of different lines in the text editor, all synthesized
    with the goal of exceeding the 64 byte block limit by a considerable amount.
    To that end, this text contains considerable excessive verbiage.";

    #[test]
    fn test_long_write() {
        assert_eq!(265, LONG_DATA.len());
        let mut sys = make_small_fs();
        let f1 = sys.open_create("one.txt").unwrap();
        sys.write(f1, LONG_DATA.as_bytes()).unwrap();
        sys.close(f1);
        let read = read_to_string(&mut sys, "one.txt");
        assert_eq!(read.as_str(), LONG_DATA);
    }

    fn read_to_string(
        sys: &mut FileSystem<16, BLOCK_SIZE, 255, 8, 512, 32, 8>,
        filename: &str,
    ) -> String {
        let fd = sys.open_read(filename).unwrap();
        let mut read = String::new();
        let mut buffer = [0; 10];
        loop {
            let num_bytes = sys.read(fd, &mut buffer).unwrap();
            let s = core::str::from_utf8(&buffer[0..num_bytes]).unwrap();
            read.push_str(s);
            if num_bytes < buffer.len() {
                sys.close(fd).unwrap();
                return read;
            }
        }
    }

    #[test]
    fn test_complex_1() {
        let one = "This is a message, a short message, but an increasingly long message.
        This is a message, a short message, but an increasingly long message.";
        let two = "This is the second message I have chosen to undertake in this particular test.
        This is a continuation of this ever-so-controversial second message.\n";
        let mut sys = make_small_fs();
        let f1 = sys.open_create("one.txt").unwrap();
        sys.write(f1, one[0..one.len() / 2].as_bytes()).unwrap();
        let f2 = sys.open_create("two.txt").unwrap();
        sys.write(f2, two[0..two.len() / 2].as_bytes()).unwrap();
        sys.write(f1, one[one.len() / 2..one.len()].as_bytes())
            .unwrap();
        sys.write(f2, two[two.len() / 2..two.len()].as_bytes())
            .unwrap();
        sys.close(f1).unwrap();
        sys.close(f2).unwrap();
        assert_eq!(one, read_to_string(&mut sys, "one.txt").as_str());
        assert_eq!(two, read_to_string(&mut sys, "two.txt").as_str());
    }

    #[test]
    fn test_complex_2() {
        let one = "This is a message, a short message, but an increasingly long message.
        This is a message, a short message, but an increasingly long message.";
        let two = "This is the second message I have chosen to undertake in this particular test.
        This is a continuation of this ever-so-controversial second message.\n";
        let mut sys = make_small_fs();
        let f1 = sys.open_create("one.txt").unwrap();
        sys.write(f1, one[0..one.len() / 2].as_bytes()).unwrap();
        let f2 = sys.open_create("two.txt").unwrap();
        sys.write(f2, two[0..two.len() / 2].as_bytes()).unwrap();
        sys.close(f1).unwrap();
        sys.close(f2).unwrap();

        let f3 = sys.open_append("two.txt").unwrap();
        let f4 = sys.open_append("one.txt").unwrap();
        sys.write(f4, one[one.len() / 2..one.len()].as_bytes())
            .unwrap();
        sys.write(f3, two[two.len() / 2..two.len()].as_bytes())
            .unwrap();
        sys.close(f1).unwrap();
        sys.close(f2).unwrap();
        assert_eq!(one, read_to_string(&mut sys, "one.txt").as_str());
        assert_eq!(two, read_to_string(&mut sys, "two.txt").as_str());
    }
    
    #[test]
    fn test_complex_3() {
        let one = "This is a message, a short message, but an increasingly long message.
        This is a message, a short message, but an increasingly long message.";
        let two = "This is the second message I have chosen to undertake in this particular test.
        This is a continuation of this ever-so-controversial second message.\n";
        let mut sys = make_small_fs();
        let f1 = sys.open_create("one.txt").unwrap();
        sys.write(f1, one.as_bytes()).unwrap();
        sys.close(f1).unwrap();

        let f2 = sys.open_create("one.txt").unwrap();
        sys.write(f2, two.as_bytes()).unwrap();
        sys.close(f2).unwrap();

        assert_eq!(two, read_to_string(&mut sys, "one.txt").as_str());
    }

    #[test]
    fn test_file_not_found() {
        let mut sys = make_small_fs();
        let f1 = sys.open_create("one.txt").unwrap();
        sys.write(f1, "This is a test.".as_bytes()).unwrap();
        sys.close(f1).unwrap();
        match sys.open_read("one.tx") {
            FileSystemResult::Ok(_) => panic!("Shouldn't have found the file"),
            FileSystemResult::Err(e) => assert_eq!(e, FileSystemError::FileNotFound),
        }
    }

    #[test]
    fn test_file_not_open() {
        let mut sys = make_small_fs();
        let f1 = sys.open_create("one.txt").unwrap();
        sys.write(f1, "This is a test.".as_bytes()).unwrap();
        sys.close(f1).unwrap();
        let fd = sys.open_read("one.txt").unwrap();
        let mut buffer = [0; 10];
        match sys.read(fd + 1, &mut buffer) {
            FileSystemResult::Ok(_) => panic!("Should be an error!"),
            FileSystemResult::Err(e) => assert_eq!(e, FileSystemError::FileNotOpen),
        }
    }

    #[test]
    fn test_not_open_for_read() {
        let mut sys = make_small_fs();
        let f1 = sys.open_create("one.txt").unwrap();
        sys.write(f1, "This is a test.".as_bytes()).unwrap();
        let mut buffer = [0; 10];
        match sys.read(f1, &mut buffer) {
            FileSystemResult::Ok(_) => panic!("Should not work!"),
            FileSystemResult::Err(e) => assert_eq!(e, FileSystemError::NotOpenForRead),
        }
    }

    #[test]
    fn test_not_open_for_write() {
        let mut sys = make_small_fs();
        let f1 = sys.open_create("one.txt").unwrap();
        sys.write(f1, "This is a test.".as_bytes()).unwrap();
        sys.close(f1).unwrap();
        let f2 = sys.open_read("one.txt").unwrap();
        match sys.write(f2, "this is also a test".as_bytes()) {
            FileSystemResult::Ok(_) => panic!("Should be an error"),
            FileSystemResult::Err(e) => assert_eq!(e, FileSystemError::NotOpenForWrite),
        }
    }

    #[test]
    fn test_filename_too_long() {
        let mut sys = make_small_fs();
        match sys.open_create("this_is_an_exceedingly_long_filename_to_use.txt") {
            FileSystemResult::Ok(_) => panic!("This should be an error"),
            FileSystemResult::Err(e) => assert_eq!(e, FileSystemError::FilenameTooLong),
        }
    }

    #[test]
    fn test_already_open() {
        let mut sys = make_small_fs();
        let f1 = sys.open_create("one.txt").unwrap();
        sys.write(f1, "This is a test.".as_bytes()).unwrap();
        match sys.open_read("one.txt") {
            FileSystemResult::Ok(_) => panic!("Should be an error"),
            FileSystemResult::Err(e) => assert_eq!(e, FileSystemError::AlreadyOpen),
        }
    }

    #[test]
    fn test_file_too_big() {
        let mut sys = make_small_fs();
        let f1 = sys.open_create("one.txt").unwrap();
        for _ in 0..sys.max_file_size() - 1 {
            sys.write(f1, "A".as_bytes()).unwrap();
        }
        match sys.write(f1, "B".as_bytes()) {
            FileSystemResult::Ok(_) => panic!("Should be an error!"),
            FileSystemResult::Err(e) => assert_eq!(e, FileSystemError::FileTooBig),
        }
    }

    #[test]
    fn test_too_many_files() {
        let mut sys = make_small_fs();
        for i in 0..MAX_FILES_STORED - 1 {
            let filename = format!("file{i}");
            let f = sys.open_create(filename.as_str()).unwrap();
            let content = format!("This is sentence {i}");
            sys.write(f, content.as_bytes()).unwrap();
            sys.close(f).unwrap();
        }
        match sys.open_create("Final") {
            FileSystemResult::Ok(_) => panic!("This should be an error!"),
            FileSystemResult::Err(e) => assert_eq!(e, FileSystemError::TooManyFiles),
        }
    }

    #[test]
    fn test_disk_full() {
        let mut sys = make_small_fs();
        for i in 0..MAX_FILES_STORED - 1 {
            let filename = format!("file{i}");
            let f = sys.open_create(filename.as_str()).unwrap();
            for j in 0..sys.max_file_size() - 1 {
                match sys.write(f, "A".as_bytes()) {
                    FileSystemResult::Ok(_) => {}
                    FileSystemResult::Err(e) => {
                        assert_eq!(i, 30);
                        assert_eq!(j, 191);
                        assert_eq!(e, FileSystemError::DiskFull);
                        return;
                    }
                }
            }
            sys.close(f).unwrap();
        }
        panic!("The disk should have been full!");
    }
}
