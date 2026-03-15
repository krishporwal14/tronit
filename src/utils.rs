use flate2::Compression;
use flate2::{read::ZlibDecoder, write::ZlibEncoder};
use sha1::{Digest, Sha1};

use std::io::{Read, Write};

pub fn sha1_hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);

    let result = hasher.finalize();

    hex::encode(result)
}

pub fn compress(data: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());

    encoder.write_all(data).unwrap();
    encoder.finish().unwrap()
}

pub fn decompress(data: &[u8]) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(data);

    let mut out = Vec::new();

    decoder.read_to_end(&mut out).unwrap();

    out
}
