use ring::hkdf::{KeyType, Prk, Salt, HKDF_SHA256};
use ring::hmac::{self, Tag};
use sha2::{Digest, Sha256};
use aes::Aes128;
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::cipher::generic_array::GenericArray;

pub(crate) static INITIAL_SALT: [u8; 20] = [
    0x38, 0x76, 0x2c, 0xf7, 0xf5, 0x59, 0x34, 0xb3, 0x4d, 0x17,
    0x9a, 0xe6, 0xa4, 0xc8, 0x0c, 0xad, 0xcc, 0xbb, 0x7f, 0x0a,
];


#[inline(always)]
pub(crate) fn build_key(salt: &[u8]) -> hmac::Key {
    hmac::Key::new(hmac::HMAC_SHA256, salt)
}

#[inline(always)]
pub(crate) fn hkdf_extract(key: &hmac::Key, ikm: &[u8], out: &mut [u8; 32]) {
    let tag = hmac::sign(key, ikm);
    out.copy_from_slice(tag.as_ref());
}

#[inline(always)]
pub(crate) fn hkdf_expand(prk: &[u8; 32], info: &[u8], out: &mut [u8]) {
    let mut t = [0u8; 32];
    let mut prev_len = 0;
    let mut ctr = 1u8;
    let mut offset = 0;

    while offset < out.len() {
        let mut h = Sha256::new();
        h.update(prk);
        if prev_len > 0 {
            h.update(&t[..prev_len]);
        }
        h.update(info);
        h.update(&[ctr]);
        t = h.finalize().into();
        let to_copy = usize::min(out.len() - offset, 32);
        out[offset..offset + to_copy].copy_from_slice(&t[..to_copy]);
        prev_len = 32;
        offset += to_copy;
        ctr = ctr.wrapping_add(1);
    }
}

// Generate the mask from ciphertext sample (16 bytes)
#[inline(always)]
pub(crate) fn generate_hp_mask(hp_key: &[u8; 16], sample: &[u8], hp_mask: &mut [u8; 16]) {
    let cipher = Aes128::new(GenericArray::from_slice(hp_key));
    let mut block = GenericArray::clone_from_slice(&sample[..16]);
    cipher.encrypt_block(&mut block);
    hp_mask.copy_from_slice(&block);
}

// Unmask the protected bits from the first byte and packet number bytes
#[inline(always)]
pub(crate) fn remove_header_protection(first_byte: &mut u8, packet_number: &mut [u8], mask: &[u8; 16]) {
    // First byte: XOR lowest 4 bits with mask[0]'s lowest 4 bits
    *first_byte ^= mask[0] & 0b1111;

    // Packet number bytes: XOR each byte with subsequent mask bytes
    for i in 0..packet_number.len() {
        packet_number[i] ^= mask[i + 1];
    }
}

#[inline(always)]
pub(crate) fn aes_ecb_encrypt(key: &[u8; 16], sample: &[u8], out: &mut [u8; 16]) {
    let cipher = Aes128::new_from_slice(key).unwrap();
    let mut block = GenericArray::<u8, _>::clone_from_slice(sample);
    cipher.encrypt_block(&mut block);
    out.copy_from_slice(&block);
}

#[inline(always)]
pub(crate) fn hkdf_expand_label(prk: &Prk, label: &[u8], out: &mut [u8]) {
    const PREFIX: &[u8] = b"tls13 ";
    let len = out.len() as u16;
    let mut info = Vec::with_capacity(2 + 1 + PREFIX.len() + label.len() + 1);
    info.extend_from_slice(&len.to_be_bytes());
    info.push((PREFIX.len() + label.len()) as u8);
    info.extend_from_slice(PREFIX);
    info.extend_from_slice(label);
    info.push(0);                             // zero-length context

    struct Len16;
    impl KeyType for Len16 { fn len(&self) -> usize { 16 } }

    prk.expand(&[&info], Len16).unwrap().fill(out).unwrap();
}

// --- helper: single AES-ECB block ---
#[inline(always)]
pub(crate) fn aes_ecb_block(key: &[u8;16], sample: &[u8;16]) -> [u8;16] {
    let cipher = Aes128::new_from_slice(key).unwrap();
    let mut block = GenericArray::<u8, _>::clone_from_slice(sample);
    cipher.encrypt_block(&mut block);
    block.into()
}

/* RFC 8446 HKDF-Expand-Label */
#[inline(always)]
pub(crate) fn expand_label(prk: &Prk, label: &[u8], out: &mut [u8]) {
    const PREFIX: &[u8] = b"tls13 ";
    let mut info = Vec::with_capacity(2 + 1 + PREFIX.len() + label.len() + 1);
    info.extend_from_slice(&(out.len() as u16).to_be_bytes());
    info.push((PREFIX.len() + label.len()) as u8);
    info.extend_from_slice(PREFIX);
    info.extend_from_slice(label);
    info.push(0); // empty context

    struct Len(usize);
    impl KeyType for Len { fn len(&self) -> usize { self.0 } }

    prk.expand(&[&info], Len(out.len())).unwrap().fill(out).unwrap();
}

#[inline(always)]
pub(crate) fn aes_block(key: &[u8; 16], sample: &[u8; 16]) -> [u8; 16] {
    let cipher = Aes128::new_from_slice(key).unwrap();
    let mut blk = GenericArray::<u8, _>::clone_from_slice(sample);
    cipher.encrypt_block(&mut blk);
    blk.into()
}

#[inline(always)]
pub(crate) fn varint(buf: &[u8]) -> Option<(usize, usize)> {
    if buf.is_empty() { return None }
    let first = buf[0];
    let (bytes, mask) = match first >> 6 { 0 => (1,0b00111111), 1 => (2,0b00111111), 2 => (4,0b00111111), _ => (8,0b00111111) };
    if buf.len() < bytes { return None }
    let mut v = (first & mask) as usize;
    for i in 1..bytes { v = (v << 8) | buf[i] as usize; }
    Some((v, bytes))
}