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
#![feature(btree_range)]
#![feature(collections_bound)]
#![feature(alloc_system)]
extern crate btrfs;
extern crate alloc_system;

use btrfs::linux::{get_file_extent_map_for_path};
use std::fs::*;
//use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;
use std::collections::BTreeMap;
use std::collections::Bound::Included;
use std::error::Error;
use std::io::Write;
use std::path::Path;


fn scan_dirs(p : &Path) -> std::io::Result<u64> {
    let mut cnt = 0;

    let mut phy_sorted : BTreeMap<u64, PathBuf> = std::collections::BTreeMap::new();
    let mut unordered : Vec<PathBuf> = vec![];
    let mut cursor = 0;

    unordered.push(p.to_owned());


    while !phy_sorted.is_empty() || !unordered.is_empty() {

        let next = {
            let next_key = phy_sorted.range((Included(&cursor), Included(&std::u64::MAX))).next().map(|(k,_)| *k);
            if let Some(k) = next_key  {
                phy_sorted.remove(&k)
            } else {
                unordered.pop()
            }
        };

        match next {
            Some(p) => {

                match read_dir(&p) {
                    Ok(dir_iter) => {
                        for de in dir_iter.filter_map(|de| de.ok()) {
                            let entry = de.path();
                            let meta = de.file_type().unwrap();
                            if meta.is_file() {
                                cnt+=1;
                            }
                            if !meta.is_dir() {
                                continue;
                            }

                            //print!{"{} {} ", entry.to_string_lossy(), meta.st_ino()};
                            match get_file_extent_map_for_path(&entry) {
                                Ok(ref extents) if !extents.is_empty() => {
                                    //println!("{:?}", extents);
                                    if let Some(old) = phy_sorted.insert(extents[0].physical, entry) {
                                        unordered.push(old);
                                    }
                                },
                                _ => {
                                    unordered.push(entry);
                                }
                            }
                        }
                    }
                    Err(open_err) => {
                        writeln!(std::io::stderr(), "skipping {} reason: {}", &p.to_string_lossy(), open_err.description())?;
                    }
                }


            },
            None => {
                cursor = 0;
            }
        }
    }


    Ok(cnt)
}

fn process_args() -> std::io::Result<u64> {
    let root = if let Some(str) = std::env::args().nth(1) {
        Path::new(&str).to_owned()
    } else {
        std::env::current_dir()?
    };
    scan_dirs(&root)
}


fn main() {

    match process_args() {
        Ok(cnt) => {println!("{}", cnt);}
        Err(e) => {writeln!(std::io::stderr(),"{}", e.description()).unwrap();}
    };
}