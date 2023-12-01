use std::env::args;
use std::process::exit;

fn main() {
    let (exit_code, message) = app(args());
    if let Some(message) = message {
        eprintln!("{}", message);
    }
    exit(exit_code);
}

fn app<I, T>(args: I) -> (i32, Option<String>)
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let mut args = args.into_iter();
    let app = args.next().unwrap();
    // We need this split into two lines, otherwise we bump into the "creates a temporary which is
    // freed while still in use" error.
    let args: Vec<_> = args.collect();
    let args: Vec<_> = args.iter().map(|item| item.as_ref()).collect();
    match args.as_slice() {
        [] => (1, Some(help_text(&app))),
        ["--help"] => (0, Some(help_text(&app))),
        ["--version"] => (0, Some(help_text(&app))),
        ["is-tagged", directory] => match cachedir::is_tagged(directory) {
            Err(e) => (2, Some(e.to_string())),
            Ok(is_tagged) => match is_tagged {
                true => (
                    0,
                    Some(format!("{} is tagged with CACHEDIR.TAG", directory)),
                ),
                false => (
                    1,
                    Some(format!("{} is not tagged with CACHEDIR.TAG", directory)),
                ),
            },
        },
        _ => (1, Some(help_text(&app))),
    }
}

fn help_text<T: AsRef<str>>(binary: T) -> String {
    let binary = binary.as_ref();
    format!(
        "Usage:
{} --help               Print this help message
{} is-tagged DIRECTORY  Check if the directory is tagged or not

Application version: 0.3.0
",
        binary, binary,
    )
}

#[test]
fn help_works() {
    assert!(app(vec!["binary", "--help"]) == (0, Some(help_text("binary"))));
    assert!(app(vec!["binary"]) == (1, Some(help_text("binary"))));
}

#[test]
fn is_tagged_works() {
    let directory = tempfile::tempdir().unwrap();
    let directory_str = directory.path().to_str().unwrap().to_string();
    assert!(
        app(vec!["binary", "is-tagged", &directory_str])
            == (
                1,
                Some(format!("{} is not tagged with CACHEDIR.TAG", directory_str))
            )
    );

    cachedir::add_tag(&directory).unwrap();
    assert!(
        app(vec!["binary", "is-tagged", &directory_str])
            == (
                0,
                Some(format!("{} is tagged with CACHEDIR.TAG", directory_str))
            )
    );

    directory.close().unwrap();

    // The directory doesn't exist anymore – we should be getting an error.
    let (exit_code, _output) = app(vec!["binary", "is-tagged", &directory_str]);
    assert!(exit_code != 0);
}
