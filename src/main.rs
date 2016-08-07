extern crate fuse;
extern crate libc;
extern crate time;
extern crate hyper;
extern crate rustc_serialize;

use std::path::Path;
use std::env;
use std::io::prelude::Read;
use libc::{ENOENT, ENOSYS};
use time::Timespec;
use fuse::{FileAttr, FileType, Filesystem, Request, ReplyAttr, ReplyData, ReplyEntry, ReplyDirectory};
use hyper::client::{Client, Response};
use rustc_serialize::json::Json;

struct RedditFileSystem;

impl Filesystem for RedditFileSystem {
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
        if ino == 1 {
            if offset == 0 {
                let subreddit = match env::args().nth(2) {
                    Some(subreddit) => subreddit,
                    None => "all".to_string()
                };

                let url = format!("https://www.reddit.com/r/{}.json", subreddit);
                println!("{}", url);
                let mut body = String::new();
                Client::new()
                    .get(&url)
                    .send()
                    .expect("Couldn't load reddit")
                    .read_to_string(&mut body);

                println!("{}", body);

                reply.add(1, 0, FileType::Directory, &Path::new("."));
                reply.add(1, 1, FileType::Directory, &Path::new(".."));
            }
            
            reply.ok();
        } else {
            reply.error(ENOSYS)
        }
    }
}

fn main() {
    let mountpoint = match env::args().nth(1) {
        Some(path) => path,
        None => {
            println!("Usage: {} <Mountpoint>", env::args().nth(0).unwrap());
            return;
        }
    };
    fuse::mount(RedditFileSystem, &mountpoint, &[])
}
