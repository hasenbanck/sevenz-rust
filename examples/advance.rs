use std::{collections::HashMap, env::temp_dir, time::Instant};

use lzma::LZMA2Options;
use rand::Rng;
use sevenz_rust2::*;

fn main() {
    let temp_dir = temp_dir();
    let src = temp_dir.join("compress/advance");
    if src.exists() {
        let _ = std::fs::remove_dir_all(&src);
    }
    let _ = std::fs::create_dir_all(&src);
    let file_count = 100;
    let mut contents = HashMap::with_capacity(file_count);
    let mut unpack_size = 0;
    // generate random content files
    {
        for i in 0..file_count {
            let c = gen_random_contents(rand::rng().random_range(1024..10240));
            unpack_size += c.len();
            contents.insert(format!("file{i}.txt"), c);
        }
        for (filename, content) in contents.iter() {
            let _ = std::fs::write(src.join(filename), content);
        }
    }
    let dest = temp_dir.join("compress/compress.7z");

    let time = Instant::now();
    // start compress
    let mut sz = SevenZWriter::create(&dest).expect("create writer ok");
    sz.set_encrypt_header(true);
    #[cfg(feature = "aes256")]
    {
        sz.set_content_methods(vec![
            AesEncoderOptions::new("sevenz-rust".into()).into(),
            LZMA2Options::with_preset(9).into(),
        ]);
        // sz.set_encrypt_header(true);
    }
    sz.push_source_path(&src, |_| true).expect("pack ok");
    println!("finish");
    sz.finish().expect("compress ok");
    println!("compress took {:?}/{:?}", time.elapsed(), dest);
    if src.exists() {
        let _ = std::fs::remove_dir_all(&src);
    }
    assert!(dest.exists());
    let dest_file = std::fs::File::open(&dest).unwrap();
    let m = dest_file.metadata().unwrap();
    println!("src  file len:{:?}", unpack_size);
    println!("dest file len:{:?}", m.len());
    println!("ratio:{:?}", m.len() as f64 / unpack_size as f64);

    // decompress
    // let archive = Archive::open_with_password(&dest, &"sevenz-rust-".into()).unwrap();
    // println!("archive:{:?}", archive);
    let mut sz = SevenZReader::open(&dest, "sevenz-rust".into()).expect("create reader ok");
    assert_eq!(contents.len(), sz.archive().files.len());
    assert_eq!(1, sz.archive().folders.len());
    sz.for_each_entries(|entry, reader| {
        let content = std::io::read_to_string(reader)?;
        assert_eq!(content, contents[entry.name()]);
        Ok(true)
    })
    .expect("decompress ok");
    let _ = std::fs::remove_file(dest);
}

fn gen_random_contents(len: usize) -> String {
    let mut s = String::with_capacity(len);
    let mut rng = rand::rng();
    for _ in 0..len {
        let ch = rng.random_range('A'..='Z');
        s.push(ch);
    }
    s
}
