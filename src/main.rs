extern crate fuse;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate time;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

use fuse::*;
use time::Timespec;

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

struct F {
    attr: FileAttr,
    parent: u64,
    sub_entries: Vec<(String, u64)>,
    content: Vec<u8>,
}

impl F {
    fn with_attr(attr: FileAttr, parent: u64) -> Self {
        F {
            attr,
            parent,
            sub_entries: Vec::default(),
            content: Vec::default(),
        }
    }
}

struct FS {
    m: HashMap<u64, F>,
}

impl FS {
    fn new() -> Self {
        let mut m = HashMap::new();
        let root_attr = FileAttr {
            ino: 1,
            size: 4096,
            blocks: 0,
            atime: time::now().to_timespec(),
            mtime: time::now().to_timespec(),
            ctime: time::now().to_timespec(),
            crtime: time::now().to_timespec(),
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 1001,
            gid: 1001,
            rdev: 0,
            flags: 0,
        };
        m.insert(1, F::with_attr(root_attr, 1));
        FS { m }
    }
}

impl Filesystem for FS {
    fn getattr(&mut self, _: &Request, ino: u64, reply: ReplyAttr) {
        if let Some(f) = self.m.get(&ino) {
            reply.attr(&TTL, &f.attr);
            return;
        }
        reply.error(libc::ENOENT);
    }

    fn readdir(&mut self, _: &Request, ino: u64, _: u64, offset: i64, mut reply: ReplyDirectory) {
        if let Some(f) = self.m.get(&ino) {
            if offset == 0 {
                reply.add(ino, 0, FileType::Directory, ".");
                reply.add(f.parent, 1, FileType::Directory, "..");
                for (i, entry) in f.sub_entries.iter().enumerate() {
                    let f = self.m.get(&entry.1).unwrap();
                    reply.add(entry.1, (i + 2) as i64, f.attr.kind, &entry.0);
                }
            }
            reply.ok();
            return;
        }
        reply.error(libc::ENOENT);
    }

    fn lookup(&mut self, _: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent != 1 {
            debug!("parent is not 1");
            // Not supported.
            reply.error(libc::ENOENT);
            return;
        }
        let f = self.m.get(&parent).unwrap();
        debug!("sub entries len: {}", f.sub_entries.len());
        for entry in &f.sub_entries {
            debug!("want: {:?}, scan: {:?}", name.to_str().unwrap(), entry.0);
            if &entry.0 == name.to_str().unwrap() {
                debug!("found {:?} when lookup", name.to_str());
                reply.entry(&TTL, &f.attr, 0);
                return;
            }
        }
        reply.error(libc::ENOENT);
    }

    fn create(
        &mut self,
        _: &Request,
        parent: u64,
        name: &OsStr,
        mode: u32,
        flags: u32,
        reply: ReplyCreate,
    ) {
        if parent != 1 {
            // Not supported.
            reply.error(libc::ENOENT);
            return;
        }
        let name = name.to_str().unwrap().to_owned();
        let next_ino = self.m.len() as u64 + 1; // The first is 1 instead of 0!

        if let Some(f) = self.m.get_mut(&parent) {
            assert!(f.attr.kind == FileType::Directory);
            f.sub_entries.push((name.clone(), next_ino));
        } else {
            reply.error(libc::ENOENT);
            return;
        }

        let attr = FileAttr {
            ino: next_ino,
            size: 0,
            blocks: 0,
            atime: time::now().to_timespec(),
            mtime: time::now().to_timespec(),
            ctime: time::now().to_timespec(),
            crtime: time::now().to_timespec(),
            kind: FileType::RegularFile,
            perm: mode as u16,
            nlink: 1,
            uid: 1001,
            gid: 1001,
            rdev: 0,
            flags: flags,
        };
        self.m.insert(next_ino, F::with_attr(attr, parent));
        reply.created(&TTL, &self.m[&next_ino].attr, 0, 0, 0);
        debug!("create {} success", name);
    }

    // fn read(
    //     &mut self,
    //     _req: &Request,
    //     ino: u64,
    //     _fh: u64,
    //     offset: i64,
    //     _size: u32,
    //     reply: ReplyData,
    // ) {
    //     if ino == 2 {
    //         reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
    //     } else {
    //         reply.error(ENOENT);
    //     }
    // }
}

fn main() {
    env_logger::init();
    let options = ["-s", "-d", "-f", "-o", "allow_other"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuse::mount(FS::new(), &PathBuf::from("a"), &options).unwrap();
}
