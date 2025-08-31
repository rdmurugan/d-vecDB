use byteorder::{ByteOrder, LittleEndian};

pub fn normalize_blob_inplace_f32(buf: &mut [u8]) {
    let n = buf.len()/4; let mut norm = 0f32;
    for i in 0..n { let v = LittleEndian::read_f32(&buf[i*4..i*4+4]); norm += v*v; }
    let s = norm.sqrt(); if s < 1e-30 { return; }
    for i in 0..n {
        let v = LittleEndian::read_f32(&buf[i*4..i*4+4]);
        LittleEndian::write_f32(&mut buf[i*4..i*4+4], v/s);
    }
}