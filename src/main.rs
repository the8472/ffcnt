//   ffcnt
//   Copyright (C) 2017 The 8472
//
//   This program is free software; you can redistribute it and/or modify
//   it under the terms of the GNU General Public License as published by
//   the Free Software Foundation; either version 3 of the License, or
//   (at your option) any later version.
//
//   This program is distributed in the hope that it will be useful,
//   but WITHOUT ANY WARRANTY; without even the implied warranty of
//   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//   GNU General Public License for more details.
//
//   You should have received a copy of the GNU General Public License
//   along with this program; if not, write to the Free Software Foundation,
//   Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301  USA
#![deny(warnings)]
#![cfg_attr(feature = "system_alloc", feature(alloc_system))]
#[cfg(feature = "system_alloc")]
extern crate alloc_system;
#[macro_use] extern crate clap;
#[macro_use] extern crate derive_error;
extern crate platter_walk;
extern crate isatty;

use std::error::Error;
use std::io::Write;
use std::path::Path;
use clap::{Arg, App};
use platter_walk::*;
use std::fs::FileType;
use std::os::unix::fs::FileTypeExt;


#[derive(Debug, Error)]
enum CliError {
    Io(std::io::Error)
}

#[derive(Copy, Clone)]
enum FileTypeMatcher {
    Dir,
    Regular,
    Symlink,
    Block,
    Char,
    Pipe,
    Socket
}

use FileTypeMatcher::*;

impl FileTypeMatcher {
    fn from(c : char) -> FileTypeMatcher {
        match c {
            'b' => Block,
            'c' => Char,
            'd' => Dir,
            'p' => Pipe,
            'f' => Regular,
            'l' => Symlink,
            's' => Socket,
            _ => panic!("invalid input")

        }
    }

    fn is(&self, ft: &FileType) -> bool {
        match *self {
            Block => ft.is_block_device(),
            Char => ft.is_char_device(),
            Pipe => ft.is_fifo(),
            Socket => ft.is_socket(),
            Symlink => ft.is_symlink(),
            Regular => ft.is_file(),
            Dir => ft.is_dir(),
        }
    }
}


type Counts = (u64, u64);


fn process_args() -> std::result::Result<Counts, CliError> {
    let matches = App::new("fast file counting")
        .version(crate_version!())
        .arg(Arg::with_name("ord").long("leaf-order").required(false).takes_value(true).possible_values(&["inode","content", "dentry"]).help("optimize order for listing/stat/reads"))
        .arg(Arg::with_name("type").long("type").required(false).takes_value(true).possible_values(&["f", "l", "d", "s","b","c", "p"]).help("filter type"))
        .arg(Arg::with_name("list").long("ls").required(false).takes_value(false).help("list files"))
        .arg(Arg::with_name("size").short("s").required(false).takes_value(false).help("sum apparent length of matched files. Implies --leaf-order inode."))
        .arg(Arg::with_name("dirs").index(1).multiple(true).required(false).help("directories to traverse [default: cwd]"))
        .arg(Arg::with_name("prefetch").long("prefetch").takes_value(false).required(false).help("attempt to prefetch directory indices from underlying mount device. requires read permission on device"))
        .get_matches();

    let mut starting_points = matches.values_of_os("dirs").map(|it| it.map(Path::new).map(Path::to_owned).collect()).unwrap_or(vec![]);
    let want_size = matches.is_present("size");
    let list = matches.is_present("list");
    let prefetch = matches.is_present("prefetch");
    let type_filter = matches.value_of("type").map(|t| FileTypeMatcher::from(t.chars().next().unwrap()));

    if starting_points.is_empty() {
        starting_points.push(std::env::current_dir()?);
    }

    let mut dir_scanner = ToScan::new();

    dir_scanner.prefetch_dirs(prefetch);

    if want_size {
        dir_scanner.set_order(Order::Inode);
    }

    match matches.value_of("ord") {
        Some("inode") => {dir_scanner.set_order(Order::Inode);},
        Some("content") => {dir_scanner.set_order(Order::Content);}
        Some("dentry") => {dir_scanner.set_order(Order::Dentries);}
        _ => {}
    };

    for path in starting_points {
        if path.is_absolute() {
            dir_scanner.add_root(path)?;
        } else {
            dir_scanner.add_root(path.canonicalize()?)?;
        }

    }

    if let Some(ref tf) = type_filter {
        let owned = tf.clone();
        dir_scanner.set_prefilter(Box::new(move |_,ft| owned.is(ft)));
    }

    let mut result = (0,0);

    for entry in dir_scanner {
        match entry  {
            Ok(e) => {
                if let Some(ref tf) = type_filter {
                    if !tf.is(&e.file_type()) {
                        continue;
                    }
                }

                if list {
                    println!("{}", e.path().to_string_lossy());
                }

                result.0 += 1;
                if want_size {
                    result.1 += e.path().metadata().unwrap().len();
                }
            }
            Err(e) => {
                writeln!(std::io::stderr(),"{}", e.description()).unwrap();
            }
        }

    }

    if !(list && isatty::stdout_isatty()) {
        println!("files: {}", result.0);
    }

    if want_size {
        println!("bytes: {}", result.1);
    }

    Ok(result)
}


fn main() {

    match process_args() {
        Ok(_) => {
            std::process::exit(0);
        }
        Err(e) => {
            writeln!(std::io::stderr(),"{}", e.description()).unwrap();
            std::io::stderr().flush().unwrap();
            std::process::exit(1);
        }
    };
}