extern crate fuse;
extern crate libc;
extern crate time;

use std::path::Path;
use std::env;
use libc::{ENOENT, ENOSYS};
use time::Timespec;
use fuse::{FileAttr, FileType, Filesystem, Request, ReplyAttr, ReplyData, ReplyEntry, ReplyDirectory};

struct RedditFileSystem;

impl Filesystem for RedditFileSystem {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr(ino={})", ino);
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
