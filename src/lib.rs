pub mod ignore;

use std::io;

use io::Write;

use std::ffi::OsStr;

use std::fs::FileType;
use std::fs::Metadata;

use std::path::Path;
use std::path::PathBuf;

pub trait Entry {
    fn path(&self) -> &Path;
    fn into_path(self) -> PathBuf;
    fn is_symlink(&self) -> bool;
    fn metadata(&self) -> Result<Metadata, io::Error>;
    fn file_type(&self) -> Option<FileType>;
    fn file_name(&self) -> &OsStr;
    fn depth(&self) -> usize;
    fn ino(&self) -> Option<u64>;
}

pub fn write_dirents<I, W, S, E>(mut wtr: W, ser: S) -> impl FnMut(I) -> Result<(), io::Error>
where
    W: Write,
    S: Fn(&E, &mut Vec<u8>) -> Result<(), io::Error>,
    E: Entry,
    I: Iterator<Item = Result<E, io::Error>>,
{
    move |dirents: I| {
        let mut buf: Vec<u8> = vec![];
        for rdirent in dirents {
            let dirent: E = rdirent?;
            buf.clear();
            ser(&dirent, &mut buf)?;
            wtr.write_all(&buf)?;
        }
        wtr.flush()
    }
}
