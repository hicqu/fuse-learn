extern crate rand;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::thread;
use std::time::Duration;

fn main() {
    let mut data = Vec::with_capacity(1 << 20);
    (0..1 << 20).for_each(|_| data.push(b'a'));

    let mut f = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open("a/x")
        .unwrap();
    f.write_all(&data).unwrap();

    let mut buf = vec![0; 4096];
    loop {
        let offset = rand::random::<u64>() % (1 << 20);
        f.seek(SeekFrom::Start(offset)).unwrap();
        f.read(&mut buf).unwrap();
        thread::sleep(Duration::from_millis(100));
    }
}
