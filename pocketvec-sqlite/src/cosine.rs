use byteorder::{ByteOrder, LittleEndian};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CosineErr {
    #[error("vector length mismatch")] Length,
    #[error("blob sizes not multiple of 4")] Align,
}

pub fn cosine_blob_f32(a: &[u8], b: &[u8]) -> Result<f32, CosineErr> {
    if a.len() % 4 != 0 || b.len() % 4 != 0 { return Err(CosineErr::Align); }
    let na = a.len() / 4; let nb = b.len() / 4; if na != nb { return Err(CosineErr::Length); }

    let mut dot = 0f32; let mut na2 = 0f32; let mut nb2 = 0f32;
    for i in 0..na {
        let va = LittleEndian::read_f32(&a[i*4..i*4+4]);
        let vb = LittleEndian::read_f32(&b[i*4..i*4+4]);
        dot += va * vb; na2 += va*va; nb2 += vb*vb;
    }
    let denom = (na2.sqrt() * nb2.sqrt()).max(1e-30);
    Ok(dot / denom)
}