extern crate fuse;

use std::env;
use fuse::Filesystem;

struct RedditFileSystem;

impl Filesystem for RedditFileSystem {
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
