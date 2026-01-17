use rand::RngCore;
use ring::aead::Nonce;

pub fn generate_connection_id(len: usize) -> Vec<u8> {
    assert!(len <= 20);
    let mut cid = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut cid);
    cid
}

pub fn encode_varint_into(mut v: u64, out: &mut Vec<u8>) {
    if v <= 63 {
        out.push(v as u8);
    } else if v <= 16383 {
        out.push(0b01_00_0000 | ((v >> 8) as u8 & 0x3F));
        out.push((v & 0xFF) as u8);
    } else if v <= 1073741823 {
        out.push(0b10_00_0000 | ((v >> 24) as u8 & 0x3F));
        out.push(((v >> 16) & 0xFF) as u8);
        out.push(((v >> 8)  & 0xFF) as u8);
        out.push((v & 0xFF) as u8);
    } else {
        out.push(0b11_00_0000 | ((v >> 56) as u8 & 0x3F));
        out.push(((v >> 48) & 0xFF) as u8);
        out.push(((v >> 40) & 0xFF) as u8);
        out.push(((v >> 32) & 0xFF) as u8);
        out.push(((v >> 24) & 0xFF) as u8);
        out.push(((v >> 16) & 0xFF) as u8);
        out.push(((v >>  8) & 0xFF) as u8);
        out.push((v & 0xFF) as u8);
    }
}

pub fn make_nonce(packet_number: u32, iv: &[u8;12]) -> Nonce {
    let mut nonce = *iv;
    let pn_be = (packet_number as u64).to_be_bytes();

    for i in 0..8 {
        let j = nonce.len() - 8 + i;
        nonce[j] ^= pn_be[i];
    }
    Nonce::assume_unique_for_key(nonce)
}