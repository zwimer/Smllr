// mock filesystem for testing

use std::path::{Path, PathBuf};
use std::io;
use std::time::SystemTime;
use std::rc::Rc;
use std::collections::HashMap;
//RUST NOTE: super is the rust equivelent of .. in the filesystem.
use super::{DeviceId, File, FileType, Inode, MetaData, VFS};
use super::super::ID;

/// TestMD is the mock metadata struct. 
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestMD {
    len: u64,
    creation: SystemTime,
    kind: FileType,
    id: ID,
}
//implementation of the MetaData trait for testMD.
impl MetaData for TestMD {
    fn get_len(&self) -> u64 {
        self.len
    }
    fn get_creation_time(&self) -> io::Result<SystemTime> {
        Ok(self.creation)
    }
    fn get_type(&self) -> FileType {
        self.kind
    }
    fn get_inode(&self) -> Inode {
        Inode(self.id.inode)
    }
    fn get_device(&self) -> io::Result<DeviceId> {
        Ok(DeviceId(self.id.dev))
    }
}

/// TestFile denotes a mockfile.
/// Note that we are mocking the linux-style filesystem
/// where many things are 'files', including directories,
/// links, devices (/dev/sda might be familair).
#[derive(Debug, Clone, PartialEq)]
pub struct TestFile {
    path: PathBuf,
    contents: Option<String>,
    kind: FileType,
    inode: Inode,
    metadata: Option<TestMD>,
}

/// implementation of the File trait
/// for TestFile. 
impl File for TestFile {
    type MD = TestMD;

    fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
    fn get_inode(&self) -> io::Result<Inode> {
        Ok(self.inode)
    }
    fn get_type(&self) -> io::Result<FileType> {
        Ok(self.kind)
    }
    fn get_metadata(&self) -> io::Result<TestMD> {
        self.metadata
            .ok_or(io::Error::new(io::ErrorKind::Other, "No MD"))
    }
}

/// TestFileSystem denotes a Mock Filesystem we use instead of risking
/// our own data. 
#[derive(Debug)]
pub struct TestFileSystem {
    files: HashMap<PathBuf, TestFile>,
    symlinks: HashMap<PathBuf, (TestFile, PathBuf)>,
}

impl TestFileSystem {
    // private helper functions:
    // gets the number of files on the mock system.
    // The name denotes its use when adding a new inode,
    // as sequentially, they are numbered 0, 1, ...
    // Ergo with N inodes, the next inode will be
    // given the id N.
    fn get_next_inode(&self) -> Inode {
        Inode((self.files.len() + self.symlinks.len()) as u64)
    }
    // Creates a new MockFile with FileType kind and a Path of path
    // Not used to create a new symlink.
    fn create_regular(&mut self, path: &Path, kind: FileType) {
        let inode = self.get_next_inode();
        // Create the metadata for the file
        let md = TestMD {
            len: 0,
            creation: SystemTime::now(),
            kind,
            id: ID {
                inode: inode.0,
                dev: 0,
            },
        };
        // Create the File.
        let tf = TestFile {
            path: path.to_owned(),
            kind,
            inode,
            contents: None,
            metadata: Some(md),
        };
        // Add the file to the filesystem.
        self.files.insert(path.to_owned(), tf);
    }

    /// constructor: initializes self. 
    pub fn new() -> Rc<Self> {
        Rc::new(TestFileSystem {
            files: HashMap::new(),
            symlinks: HashMap::new(),
        })
    }
   /// Creates a new file at path. Anologous to '$touch path'
    pub fn create_file<P: AsRef<Path>>(&mut self, path: P) {
        self.create_regular(path.as_ref(), FileType::File);
    }
    /// Creates a new directory with path. Anologus to '$mkdir path'
    pub fn create_dir<P: AsRef<Path>>(&mut self, path: P) {
        self.create_regular(path.as_ref(), FileType::Dir);
    }
    /// Creates a new symlink from path to target. analogous to 
    /// '$ln -s -t target path
    pub fn create_symlink<P: AsRef<Path>>(&mut self, path: P, target: P) {
        // Create the symlink file.
        let tf = TestFile {
            path: path.as_ref().to_owned(),
            kind: FileType::Symlink,
            inode: self.get_next_inode(),
            contents: None,
            metadata: None,
        };
        // add the symlink to the filesystem.
        let val = (tf, target.as_ref().to_owned());
        self.symlinks.insert(path.as_ref().to_owned(), val);
    }

    // getters for the Mock Filesystem.
    // RUST SYNTAX: <'a> is a lifetime paramater. Lifetimes are pretty
    // unique to rust; essentially they are used to pass the parent
    // through so they are invalidated when the parent is. 

    ///Resolves the 
    fn lookup<'a>(&'a self, path: &Path) -> io::Result<&'a TestFile> {
        if let Some(tf) = self.files.get(path) {
            Ok(tf)
        } else {
            // traverse the symlink chain
            let mut cur = self.symlinks.get(path);
            let mut seen: Vec<&Path> = vec![]; // SystemTime isn't Hash
            while let Some(c) = cur {
                if seen.contains(&c.1.as_path()) {
                    // infinite symlink loop
                    return Err(io::Error::from_raw_os_error(40));
                } else {
                    seen.push(&c.1);
                    cur = self.symlinks.get(&c.1);
                }
            }
            Err(io::Error::new(io::ErrorKind::NotFound, "No such file"))
        }
    }
}

// Implementation of the VFS interface for the whole of the Mock File System.
impl VFS for Rc<TestFileSystem> {
    type FileIter = TestFile;

    /// VFS::list_dir(p)  gets an iterator over the contents of p. 
	 fn list_dir<P: AsRef<Path>>(
        &self,
        p: P,
    ) -> io::Result<Box<Iterator<Item = io::Result<TestFile>>>> {
        let mut v = vec![];
		  // collect all files which are children of p
        for (path, file) in &self.files {
            let parent = path.parent();
            if parent == Some(p.as_ref()) || parent.is_none() {
                v.push(Ok(file.clone()));
            }
        }
		  // collect all symlinks which are children of p
        for (src, &(ref file, ref _dst)) in &self.symlinks {
            if src.parent() == Some(p.as_ref()) {
                v.push(Ok(file.clone()));
            }
        }
		  // return the iterator. 
        Ok(Box::new(v.into_iter()))
    }

//RUST NOTE: match is roughly equivlent to the c's 'switch'. 
// match expr {
//     expr1 => block,
//     expr2 => block,
// }
// is equivlent to
// switch (expr) {
//     case expr1:
//         block
//     case expr2:
//         block
//}
//
// The '_' expresion when used in match is equivelent to default in c
//
//Match also supports deconstructing and binding. see 
// https://rustbyexample.com/flow_control/match.html
// for more information.

    /// VFS::get_metadata gets the Metadata of Path 
    /// FileType of path cannot be symlink; they are handled diffrently; use
	 /// VFS::get_symlink_metadata for symlinks
    fn get_metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<<Self::FileIter as File>::MD> {
        match self.files.get(path.as_ref()) {
            Some(f) => f.get_metadata(),
            None => match self.symlinks.get(path.as_ref()) {
                Some(&(_, ref p)) => self.lookup(p).and_then(|f| f.get_metadata()),
                None => Err(io::Error::new(io::ErrorKind::NotFound, "No such file")),
            },
        }
    }

    /// VFS::get_symlink_metadata(p) gets the metadata for symlink p. 
    fn get_symlink_metadata<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> io::Result<<Self::FileIter as File>::MD> {
        match self.files.get(path.as_ref()) {
            Some(f) => f.get_metadata(),
            None => match self.symlinks.get(path.as_ref()) {
                Some(&(ref f, _)) => f.get_metadata(),
                None => Err(io::Error::new(io::ErrorKind::NotFound, "No such file")),
            },
        }
    }

    /// VFS::read_link(p) resolves symlink at path p to the path its pointing to
	 /// or gives an error if the link is broken. 
    fn read_link<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        match self.symlinks.get(path.as_ref()) {
            Some(&(_, ref p)) => Ok(p.to_owned()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "No such file")),
        }
    }
}
