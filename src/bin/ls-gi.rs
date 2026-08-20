use std::io;
use std::process::ExitCode;

use io::BufWriter;
use io::Write;

use std::path::Path;

use ignore::Walk;

use rs_ls_gi::Entry;
use rs_ls_gi::ignore::Builder;
use rs_ls_gi::ignore::Writer;

struct Config {
    dirname: String,
    single_threaded: bool,
    ignore_hidden: bool,
    ignore_dirents_by_gitignore: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dirname: ".".into(),
            single_threaded: false,
            ignore_hidden: false,
            ignore_dirents_by_gitignore: false,
        }
    }
}

pub const HELPMSG: &str = r#"
ls-gi: Recursive directory listing in LTSV format

Usage: ls-gi [OPTIONS] [DIRNAME]

Arguments:
  DIRNAME              The directory to traverse (default: ".")

Options:
  --help                       Print this help message
  --single-threaded            Use a single thread for traversal instead of multiple
  --ignore-hidden              Skip hidden files and directories (starting with '.')
  --ignore-dirents-using-gitignore 
                               Respect .gitignore rules when traversing

Output Format:
  The tool outputs Labeled Tab-Separated Values (LTSV). Each line represents one entry:
  is_softlink:<bool>	depth:<int>	ino:<uint64>	path:<string>

Example:
  ls-gi --ignore-hidden .
"#;

impl Config {
    fn from_args() -> Self {
        let mut cfg: Config = Config::default();
        for arg in std::env::args_os() {
            let raw: &[u8] = arg.as_os_str().as_encoded_bytes();
            match raw {
                b"--single-threaded" => cfg.single_threaded = true,
                b"--ignore-hidden" => cfg.ignore_hidden = true,
                b"--ignore-dirents-using-gitignore" => cfg.ignore_dirents_by_gitignore = true,
                b"--help" => {
                    println!("{HELPMSG}");
                    std::process::exit(0);
                }
                _ => cfg.dirname = arg.into_string().unwrap_or_default(),
            }
        }

        if cfg.dirname.is_empty() {
            cfg.dirname = ".".into();
        }

        cfg
    }
}

impl From<Config> for Builder {
    fn from(cfg: Config) -> Self {
        let mut bldr = Builder::from_path(&cfg.dirname);
        if cfg.single_threaded {
            bldr = bldr.single_threaded();
        }
        if cfg.ignore_hidden {
            bldr = bldr.ignore_hidden();
        }
        if cfg.ignore_dirents_by_gitignore {
            bldr = bldr.ignore_dirents_by_gitignore();
        }
        bldr
    }
}

fn dirent2ltsv<E>(dirent: &E, buf: &mut Vec<u8>) -> Result<(), io::Error>
where
    E: Entry,
{
    let is_softlink: bool = dirent.is_symlink();
    let depth: usize = dirent.depth();
    let ino: String = dirent.ino().map(|i| format!("{i}")).unwrap_or_default();

    let dpat: &Path = dirent.path();
    let spat: &str = &dpat.to_string_lossy();

    write!(buf, "is_softlink:{is_softlink}\t")?;
    write!(buf, "depth:{depth}\t")?;
    write!(buf, "ino:{ino}\t")?;
    writeln!(buf, "path:{spat}")?;

    Ok(())
}

fn sub() -> Result<(), io::Error> {
    let cfg: Config = Config::from_args();

    let bldr: Builder = cfg.into();
    let walk: Walk = bldr.build();

    let o = io::stdout();
    let mut ol = o.lock();
    {
        let bw = BufWriter::new(&mut ol);
        let mut dirents_consumer = Writer {
            wtr: bw,
            ser: dirent2ltsv,
        }
        .into_dirents_consumer();
        let mapd = walk.map(|r| r.map_err(io::Error::other));
        dirents_consumer(mapd)?;
    }
    ol.flush()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
