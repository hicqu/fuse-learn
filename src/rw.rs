extern crate rand;
use std::fs::OpenOptions;

fn main() {
    let mut data = Vec::with_capacity::<u8>(1 << 20);
    (0..1 << 20).for_each(data.push('a'));

    let mut f = OpenOptions::new().write(true).read(true).create(true).open("a/x");
    f.write_all(&data).unwrap();

    loop {
        let offset: usize = rand::random() % (1 << 20);
        f.seek(SeekFrom)
    }


    println!("hello world");
}
