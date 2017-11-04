#[cfg(test)]
mod test {

    use log::LogLevelFilter;
    use env_logger::LogBuilder;

    use std::rc::Rc;
    use std::ffi::OsStr;

    use super::super::DirWalker;
    use super::super::vfs::TestFileSystem;

    /// add to top of a test case to set the logger to ouput everything.
    // Rust note: the starting _ indicates that this might not be used.
    fn _enable_logging() {
        LogBuilder::new()
            .filter(None, LogLevelFilter::max())
            .init()
            .unwrap();
    }

    /// Test with an empty filesystem; ie, nothing to process.
    #[test]
    fn empty_fs() {
        let fs = TestFileSystem::new();
        let paths = vec![OsStr::new("/")];
        //let mut dw = DirWalker::new(fs, paths);
        //let count: usize = dw.traverse_all();
        let files = DirWalker::new(fs, paths).traverse_all();
        assert_eq!(files.len(), 0);
    }

    /// test with a single file.
    #[test]
    fn basic_fs() {
        let mut fs = TestFileSystem::new();
        {
            let fs = Rc::get_mut(&mut fs).unwrap();
            fs.create_dir("/");
            fs.create_file("/alpha");
        }
        let dw = DirWalker::new(fs, vec![OsStr::new("/")]);
        let files = dw.traverse_all();
        assert_eq!(files.len(), 1);
    }

    /// test with symlinks; includes cases for repitition and looping.
    #[test]
    fn handle_symlinks() {
        let mut fs = TestFileSystem::new();
        {
            let fs = Rc::get_mut(&mut fs).unwrap();
            fs.create_dir("/");
            fs.create_file("/alpha");
            // only deal with a target once, omit symlinks
            fs.create_symlink("/beta", "/alpha");
            fs.create_symlink("/gamma", "/alpha");
            // ignore bad symlinks
            fs.create_symlink("/delta", "/_nonexistant");
            // ignore symlink loops
            fs.create_symlink("/x", "/xx");
            fs.create_symlink("/xx", "/x");
        }
        let dw = DirWalker::new(fs, vec![OsStr::new("/")]);
        let files = dw.traverse_all();
        assert_eq!(files.len(), 1);
    }

}
