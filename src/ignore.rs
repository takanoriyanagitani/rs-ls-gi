use std::{
    ffi::OsStr,
    fs::Metadata,
    io,
    path::{Path, PathBuf},
};

use io::Write;

use ignore::DirEntry;
use ignore::Walk;
use ignore::WalkBuilder;

use crate::Entry;

impl Entry for DirEntry {
    fn path(&self) -> &Path {
        DirEntry::path(self)
    }

    fn into_path(self) -> PathBuf {
        DirEntry::into_path(self)
    }

    fn is_symlink(&self) -> bool {
        self.path_is_symlink()
    }

    fn metadata(&self) -> Result<Metadata, io::Error> {
        DirEntry::metadata(self).map_err(io::Error::other)
    }

    fn file_type(&self) -> Option<std::fs::FileType> {
        DirEntry::file_type(self)
    }

    fn file_name(&self) -> &OsStr {
        DirEntry::file_name(self)
    }

    fn depth(&self) -> usize {
        DirEntry::depth(self)
    }

    #[cfg(unix)]
    fn ino(&self) -> Option<u64> {
        DirEntry::ino(self)
    }

    #[cfg(not(unix))]
    fn ino(&self) -> Option<u64> {
        None
    }
}

pub struct Builder(pub WalkBuilder);

impl Builder {
    pub fn from_path<P>(p: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut wbld = WalkBuilder::new(p);
        wbld.hidden(false);
        wbld.git_ignore(false);
        Self(wbld)
    }
}

impl Builder {
    pub fn single_threaded(mut self) -> Self {
        self.0.threads(1);
        Self(self.0)
    }

    pub fn ignore_hidden(mut self) -> Self {
        self.0.hidden(true);
        Self(self.0)
    }

    pub fn ignore_dirents_by_gitignore(mut self) -> Self {
        self.0.git_ignore(true);
        Self(self.0)
    }

    pub fn build(&self) -> Walk {
        self.0.build()
    }
}

pub struct Writer<W, S> {
    pub wtr: W,
    pub ser: S,
}

impl<W, S> Writer<W, S>
where
    W: Write,
    S: Fn(&DirEntry, &mut Vec<u8>) -> Result<(), io::Error>,
{
    pub fn into_dirents_consumer<I>(self) -> impl FnMut(I) -> Result<(), io::Error>
    where
        I: Iterator<Item = Result<DirEntry, io::Error>>,
    {
        crate::write_dirents(self.wtr, self.ser)
    }
}
