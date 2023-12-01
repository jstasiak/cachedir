//! A Rust library to help interacting with cache directories and `CACHEDIR.TAG` files as defined
//! in [Cache Directory Tagging Specification](https://bford.info/cachedir/).
//!
//! The abstract of the spefication should be more than enough to illustrate what we're doing here:
//!
//! > Many applications create and manage directories containing cached information about content
//! > stored elsewhere, such as cached Web content or thumbnail-size versions of images or movies.
//! > For speed and storage efficiency we would often like to avoid backing up, archiving, or
//! > otherwise unnecessarily copying such directories around, but it is a pain to identify and
//! > individually exclude each such directory during data transfer operations. I propose an
//! > extremely simple convention by which applications can reliably "tag" any cache directories
//! > they create, for easy identification by backup systems and other data management utilities.
//! > Data management utilities can then heed or ignore these tags as the user sees fit.
use std::io::prelude::*;
use std::{env, fs, io, path};

/// The `CACHEDIR.TAG` file header as defined by the specification.
pub const HEADER: &[u8; 43] = b"Signature: 8a477f597d28d172789f06886806bc55";

/// Returns `true` if the tag is present at `directory`, `false` otherwise.
///
/// This is basically a shortcut for
///
/// ```ignore
/// get_tag_state(directory).map(|state| match state {
///     TagState::Present => true,
///     _ => false,
/// })
/// ```
///
/// See [get_tag_state](fn.get_tag_state.html) for error conditions documentation.
pub fn is_tagged<P: AsRef<path::Path>>(directory: P) -> io::Result<bool> {
    get_tag_state(directory).map(|state| matches!(state, TagState::Present))
}

/// Gets the state of the tag in the specified directory.
///
/// Will return an error if:
///
/// * The directory can't be accessed for any reason (it doesn't exist, permission error etc.)
/// * The `CACHEDIR.TAG` in the directory exists but can't be accessed or read from
pub fn get_tag_state<P: AsRef<path::Path>>(directory: P) -> io::Result<TagState> {
    let directory = directory.as_ref();
    match fs::File::open(directory.join("CACHEDIR.TAG")) {
        Ok(mut cachedir_tag) => {
            let mut buffer = vec![0; HEADER.len()];
            let read = cachedir_tag.read(&mut buffer)?;
            let header_ok = read == HEADER.len() && buffer == HEADER[..];
            Ok(if header_ok {
                TagState::Present
            } else {
                TagState::WrongHeader
            })
        }
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                if directory.is_dir() {
                    Ok(TagState::Absent)
                } else {
                    Err(e)
                }
            }
            _ => Err(e),
        },
    }
}

/// The state of a `CACHEDIR.TAG` file.
pub enum TagState {
    /// The file doesn't exist.
    Absent,
    /// The file exists, but doesn't contain the header required by the
    /// specification.
    WrongHeader,
    /// The file exists and contains the correct header.
    Present,
}

/// Adds a tag to the specified `directory`.
///
/// Will return an error if:
///
/// * The `directory` exists and contains a `CACHEDIR.TAG` file, regardless of its content.
/// * The file can't be created for any reason (the `directory` doesn't exist, permission error,
///   can't write to the file etc.)
pub fn add_tag<P: AsRef<path::Path>>(directory: P) -> io::Result<()> {
    let directory = directory.as_ref();
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("CACHEDIR.TAG"))
    {
        Ok(mut cachedir_tag) => cachedir_tag.write_all(HEADER),
        Err(e) => Err(e),
    }
}

/// Ensures the tag exists in `directory`.
///
/// This function considers the `CACHEDIR.TAG` file in `directory` existing, regardless of its
/// content, as a success.
///
/// Will return an error if The tag file doesn't exist and can't be created for any reason
/// (the `directory` doesn't exist, permission error, can't write to the file etc.).
pub fn ensure_tag<P: AsRef<path::Path>>(directory: P) -> io::Result<()> {
    match add_tag(directory) {
        Err(e) => match e.kind() {
            io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(e),
        },
        other => other,
    }
}

/// Tries to create `directory` with a `CACHEDIR.TAG` file atomically and returns `true` if it
/// created it or `false` if the directory already exists, regardless of if the `CACHEDIR.TAG`
/// file exists in it or if it has the correct header.
///
/// This function first creates a temporary directory in the same directory where `directory` is
/// supposed to exist. The temporary directory has a semi-random name based on the `directory` base
/// name. Then the `CACHEDIR.TAG` file is created in the temporary directory and the temporary
/// directory is attempted to be renamed to `directory`. This (as opposed to creating the directory
/// with the final name and creating `CACHEDIR.TAG` file in it) is a way to ensure that the
/// `directory` is always created with the `CACHEDIR.TAG` file. If we simply created the directory
/// with the final name the program could be interrupted before `CACHEDIR.TAG` creation and the
/// `directory` would remain not excluded from backups as this function does not attempt to verify
/// or change the `CACHEDIR.TAG` file in `directory` if it already exists.
pub fn mkdir_atomic<P: AsRef<path::Path>>(directory: P) -> io::Result<bool> {
    let mut directory = directory.as_ref().to_path_buf();
    if directory.exists() {
        return Ok(false);
    }

    if directory.is_relative() {
        directory = env::current_dir()?.join(directory);
    }

    let tempdir = tempfile::Builder::new()
        .prefix(directory.file_name().unwrap())
        .tempdir_in(directory.parent().unwrap())?;
    add_tag(tempdir.path())?;
    match fs::rename(tempdir.path(), &directory) {
        Ok(()) => Ok(true),
        Err(e) => {
            if directory.is_dir() {
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}

#[test]
fn is_tagged_on_nonexistent_directory_is_an_error() {
    let directory = path::Path::new("this directory does not exist");
    assert!(!directory.exists());
    assert!(is_tagged(directory).is_err());
}

#[test]
fn empty_directory_is_not_tagged() {
    assert!(!is_tagged(tempfile::tempdir().unwrap()).unwrap());
}

#[test]
fn directory_with_a_tag_with_wrong_content_is_not_tagged() {
    let directory = tempfile::tempdir().unwrap();
    let cachedir_tag = directory.path().join("CACHEDIR.TAG");

    fs::write(&cachedir_tag, "").unwrap();
    assert!(!is_tagged(&directory).unwrap());

    fs::write(&cachedir_tag, &HEADER[..(HEADER.len() - 2)]).unwrap();
    assert!(!is_tagged(&directory).unwrap());
}

#[test]
fn add_tag_is_detected_by_is_tagged() {
    let directory = tempfile::tempdir().unwrap();
    add_tag(directory.path()).unwrap();
    assert!(is_tagged(directory.path()).unwrap());
}

#[test]
fn add_tag_errors_when_called_with_nonexistent_directory() {
    let directory = path::Path::new("this directory does not exist");
    assert!(!directory.exists());
    assert!(add_tag(directory).is_err());
}

#[test]
fn add_tag_errors_when_tag_already_exists() {
    let directory = tempfile::tempdir().unwrap();
    assert!(add_tag(directory.path()).is_ok());
    assert!(add_tag(directory.path()).is_err());
}

#[test]
fn ensure_tag_is_detected_by_is_tagged() {
    let directory = tempfile::tempdir().unwrap();
    ensure_tag(directory.path()).unwrap();
    assert!(is_tagged(directory.path()).unwrap());
}

#[test]
fn ensure_tag_errors_when_called_with_nonexistent_directory() {
    let directory = path::Path::new("this directory does not exist");
    assert!(!directory.exists());
    assert!(ensure_tag(directory).is_err());
    assert!(is_tagged(directory).is_err());
}

#[test]
fn ensure_tag_is_idempotent() {
    let directory = tempfile::tempdir().unwrap();
    assert!(ensure_tag(directory.path()).is_ok());
    assert!(is_tagged(directory.path()).unwrap());
    assert!(ensure_tag(directory.path()).is_ok());
    assert!(is_tagged(directory.path()).unwrap());
}

#[test]
fn mkdir_atomic_works() {
    use std::thread;
    let directory = tempfile::tempdir().unwrap();
    let cache = directory.path().join("cache");
    let threads = (0..10).map(|_| {
        let cache = cache.clone();
        thread::spawn(move || mkdir_atomic(cache))
    });
    let results = threads.map(|t| t.join().unwrap().unwrap());
    let creations: usize = results.map(|created| if created { 1 } else { 0 }).sum();
    // One and only one actually creates the desired directory...
    assert_eq!(creations, 1);
    // ...which is tagged correctly.
    assert!(is_tagged(cache).unwrap());

    // The mkdir_atomic() calls which didn't actually create the final directory shouldn't leave
    // behind any garbage.
    assert_eq!(
        fs::read_dir(directory.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>(),
        ["cache"],
    );
}
