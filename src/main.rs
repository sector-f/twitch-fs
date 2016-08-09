extern crate fuse;
extern crate libc;
extern crate time;
extern crate hyper;
extern crate rustc_serialize;
extern crate clap;

use std::collections::BTreeMap;
use std::path::Path;
use std::env;
use std::io::prelude::Read;
use libc::{ENOENT, ENOSYS};
use time::Timespec;
use fuse::{FileAttr, FileType, Filesystem, Request, ReplyAttr, ReplyData, ReplyEntry, ReplyDirectory, ReplyOpen};
use hyper::client::{Client, Response};
use rustc_serialize::json::Json;
use clap::{App, Arg};

struct TwitchFileSystem {
    attrs: BTreeMap<u64, FileAttr>,
    inodes: BTreeMap<String, u64>
}

impl TwitchFileSystem {
    fn new() -> TwitchFileSystem {
        let mut attrs = BTreeMap::new();
        let mut inodes = BTreeMap::new();
        let ts = time::now().to_timespec();
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
        attrs.insert(1, attr);
        inodes.insert("/".to_owned(), 1);
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
                    let attr = FileAttr {
                        ino: i as u64 + 2,
                        size: 0,
                        blocks: 0,
                        atime: ts,
                        mtime: ts,
                        ctime: ts,
                        crtime: ts,
                        kind: FileType::RegularFile,
                        perm: 0o644,
                        nlink: 0,
                        uid: 0,
                        gid: 0,
                        rdev: 0,
                        flags: 0
                    };
                    let name = game
                        .find_path(&["game", "name"])
                        .unwrap()
                        .as_string()
                        .unwrap();

                    attrs.insert(attr.ino, attr);
                    inodes.insert(name.to_owned(), attr.ino);
                }
            },
            Err(_) => println!("Twitch returned invalid json")
        }
        TwitchFileSystem {attrs: attrs, inodes: inodes}
    }
}

impl Filesystem for TwitchFileSystem {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.attrs.get(&ino) {
            Some(attr) => {
                let ttl = Timespec::new(1, 0);
                reply.attr(&ttl, attr);
            },
            None => reply.error(ENOENT)
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, fh: u64, offset: u64, mut reply: ReplyDirectory) {
        if offset == 0 {
            for (game, &inode) in &self.inodes {
                if inode == 1 { continue; }
                let offset = inode;
                reply.add(inode, offset, FileType::RegularFile, &Path::new(game));
            }
            reply.add(1, 0, FileType::Directory, &Path::new("."));
            reply.add(1, 1, FileType::Directory, &Path::new(".."));
        }

        reply.ok();
    }
    fn read(&mut self, _req: &Request, ino: u64, fh: u64, offset: u64, size: u32, reply: ReplyData) {
        reply.data("test".as_bytes());
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
    let fs = TwitchFileSystem::new();

    fuse::mount(fs , &mountpoint, &[])
}
