extern crate fuse;
extern crate libc;
extern crate time;
extern crate hyper;
extern crate rustc_serialize;
extern crate clap;

use std::path::Path;
use std::env;
use std::io::prelude::Read;
use libc::{ENOENT, ENOSYS};
use time::Timespec;
use fuse::{FileAttr, FileType, Filesystem, Request, ReplyAttr, ReplyData, ReplyEntry, ReplyDirectory};
use hyper::client::{Client, Response};
use rustc_serialize::json::Json;
use clap::{App, Arg};

struct TwitchFileSystem;

impl Filesystem for TwitchFileSystem {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        let ts = Timespec::new(0, 0);
        let attr = FileAttr {
            ino: 1,
            size: 0,
            blocks: 0,
            atime: ts,
            mtime: ts,
            ctime: ts,
            crtime: ts,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 0,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0
        };
        let ttl = Timespec::new(1, 0);

        if ino == 1 {
            reply.attr(&ttl, &attr);
        } else {
            reply.error(ENOSYS);
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, fh: u64, offset: u64, mut reply: ReplyDirectory) {
        if offset == 0 {
            let mut body = String::new();
            Client::new()
                .get("https://api.twitch.tv/kraken/games/top")
                .send()
                .expect("Couldn't load twitch")
                .read_to_string(&mut body);

            match Json::from_str(&body) {
                Ok(data) => {
                    let games = data.find("top").unwrap().as_array().unwrap();
                    for (i, game) in games.iter().enumerate() {
                        let name = game
                            .find_path(&["game", "name"])
                            .unwrap()
                            .as_string()
                            .unwrap();
                        reply.add(1, 1 + 1 as u64,
                                  FileType::Directory,
                                  &Path::new(name));
                    }
                },
                Err(_) => println!("Twitch returned invalid json")
            }

            reply.add(1, 0, FileType::Directory, &Path::new("."));
            reply.add(1, 1, FileType::Directory, &Path::new(".."));
        }

        reply.ok();
    }
}

fn main() {
    let matches = App::new("twitch-fs")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown version"))
        .arg(Arg::with_name("mountpoint")
             .index(1)
             .required(true))
        .get_matches();

    // unwrap() is safe here because the argument is set as required
    let mountpoint = matches.value_of_os("mountpoint").unwrap();

    fuse::mount(TwitchFileSystem, &mountpoint, &[])
}
