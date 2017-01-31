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
//use std::os::unix::fs::DirEntryExt;
use std::path::PathBuf;
use std::collections::VecDeque;
use std::collections::BTreeMap;
use std::collections::Bound::Included;
use std::error::Error;
use std::io::Write;
use std::path::Path;

struct ToScan {
    phy_sorted : BTreeMap<u64, PathBuf>,
    unordered : VecDeque<PathBuf>,
    cursor: u64
}

impl ToScan {

    fn new() -> ToScan {
        ToScan{phy_sorted: BTreeMap::new(), unordered: VecDeque::new(), cursor: 0}
    }

    fn is_empty(&self) -> bool {
        self.phy_sorted.is_empty() && self.unordered.is_empty()
    }

    fn get_next(&mut self) -> Option<PathBuf> {
        if !self.unordered.is_empty() {
            return self.unordered.pop_front();
        }

        let next_key = self.phy_sorted.range((Included(&self.cursor), Included(&std::u64::MAX))).next().map(|(k,_)| *k);
        if let Some(k) = next_key {
            self.cursor = k;
            return self.phy_sorted.remove(&k);
        }

        None
    }

    fn add(&mut self, to_add : PathBuf, pos : Option<u64>) {
        match pos {
            Some(idx) => {
                if let Some(old) = self.phy_sorted.insert(idx, to_add) {
                    self.unordered.push_back(old);
                }
            }
            None => {
                self.unordered.push_back(to_add);
            }
        }
    }

    fn scan(&mut self) -> std::io::Result<u64> {
        let mut cnt = 0;

        while !self.is_empty() {
            let next = self.get_next();

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
                                        self.add(entry, Some(extents[0].physical));
                                    },
                                    _ => {
                                        //self.add(entry, Some(de.ino()))
                                        self.add(entry, None)
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
                    self.cursor = 0;
                }
            }
        }

        Ok(cnt)
    }
}


fn scan_dirs(paths : Vec<PathBuf>) -> std::io::Result<u64> {

    let mut dirs = ToScan::new();

    for p in paths {
        dirs.add(p, None);
    }

    dirs.scan()
}

fn process_args() -> std::io::Result<u64> {
    let mut starting_points = std::env::args().skip(1).map(|s| Path::new(&s).to_owned()).collect::<Vec<_>>();

    if starting_points.is_empty() {
        starting_points.push(std::env::current_dir()?);
    }

    scan_dirs(starting_points)
}


fn main() {

    match process_args() {
        Ok(cnt) => {println!("{}", cnt);}
        Err(e) => {
            writeln!(std::io::stderr(),"{}", e.description()).unwrap();
            std::io::stderr().flush().unwrap();
            std::process::exit(1);
        }
    };
}