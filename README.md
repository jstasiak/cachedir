# cachedir
[![crates.io](https://img.shields.io/crates/v/cachedir)](https://crates.io/crates/cachedir)
[![CI](https://github.com/jstasiak/cdir/workflows/CI/badge.svg)](https://github.com/jstasiak/cdir/actions?query=workflow%3ACI+branch%3Amaster)
[![codecov](https://codecov.io/gh/jstasiak/cachedir/branch/master/graph/badge.svg?token=JUWV54A0WG)](https://codecov.io/gh/jstasiak/cachedir)

A Rust library and a CLI tool to help interacting with cache directories and CACHEDIR.TAG files
as defined in [Cache Directory Tagging Specification](https://bford.info/cachedir/).

You can find the [library documentation on docs.rs](https://docs.rs/cachedir/).

To install the CLI tool run `cargo install cachedir`. To see what options are available
run `cachedir --help`. Only one subcommand, `is-tagged`, is implemented right now.
It allows checking if a directory is tagged with `CACHEDIR.TAG` (prints the relevant
information to stderr *and* sets an appropriate exit-code: 0 for true, 1 for false, 2 for an error):

```
~/projects/cachedir% ls -lah target 
total 16
drwxr-xr-x@  6 user  staff   192B Dec 10 17:02 ./
drwxr-xr-x  10 user  staff   320B Dec 10 17:16 ../
-rw-r--r--   1 user  staff   1.4K Dec 12 21:52 .rustc_info.json
-rw-r--r--   1 user  staff   177B Dec 10 15:52 CACHEDIR.TAG
drwxr-xr-x  13 user  staff   416B Dec 12 21:47 debug/
drwxr-xr-x@  5 user  staff   160B Dec 10 17:02 rls/

~/projects/cachedir% cat target/CACHEDIR.TAG 
Signature: 8a477f597d28d172789f06886806bc55
# This file is a cache directory tag created by cargo.
# For information about cache directory tags see https://bford.info/cachedir/


~/projects/cachedir% cachedir is-tagged does-not-exist
No such file or directory (os error 2)
% echo $?
2

~/projects/cachedir% cachedir is-tagged .             
. is not tagged with CACHEDIR.TAG
~/projects/cachedir% echo $?                                 
1

~/projects/cachedir% cachedir is-tagged target
target is tagged with CACHEDIR.TAG
~/projects/cachedir% echo $?
0
```
  

Versions 0.1.0 and 0.1.1 of this crate on [crates.io](https://crates.io) are actually distributions of
[a different, abandonded project by Lilian Anatolie Moraru](https://github.com/lilianmoraru/cachedir).
Credits to Lilian for transferring the name to me.
