use ring::hkdf::{Prk, Salt, HKDF_SHA256};

use super::{aes_block, expand_label, varint};
pub struct QuicLongHeader<'a> {
    pub(crate) flags: u8, // 0b1xTTXXXX
    pub(crate) version: u32,
    pub(crate) dcid: &'a [u8],
    pub(crate) scid: &'a [u8],
    pub(crate) header_size: u8,
}
impl<'a> QuicLongHeader<'a> {
/*
    pub fn parse_raw(packet: &'a [u8], initial_salt: Salt, client_init_out: &mut [u8; 32], hp_key_out: &mut [u8; 16]) -> Option<Self> {
        let dcid_len = packet[5] as usize;
        if dcid_len > 20 || 6 + dcid_len >= packet.len() { return None; }
        let dcid_end = 6 + dcid_len;

        let dcid = &packet[6..dcid_end];
        let _dbg_dcid_u64 = u64::from_be_bytes(dcid.to_vec().as_slice().try_into().unwrap_or([0; 8]));
        
        let scid_len = packet[dcid_end] as usize;
        let scid_start = dcid_end + 1;
        if scid_len > 20 || scid_start + scid_len > packet.len() { return None; }
        let scid_end = scid_start + scid_len;

        let scid = &packet[scid_start .. scid_end];
        let _dbg_scid_u64 = u64::from_be_bytes(scid.to_vec().as_slice().try_into().unwrap_or([0; 8]));
        /* derive secrets */
        let initial_secret = initial_salt.extract(dcid);
        expand_label(&initial_secret, b"client in", &mut client_init_out);
        let prk = Prk::new_less_safe(HKDF_SHA256, &client_init_out);
        expand_label(&prk, b"quic hp", &mut hp_key_out);
        /* locate packet-number offset */
        let mut off = scid_end;
        let Some((tok, tl)) = varint(&packet[off..]) else { return None; };
        off += tl + tok;
        let Some((plen, pl)) = varint(&packet[off..]) else { return None; };
        off += pl;                                            // PN offset
        if off + 20 > packet.len() { return None; }
        let sample: &[u8;16] = (&packet[off+4..off+20]).try_into().unwrap();
        let mask = aes_block(&hp_key_out, sample);
        /* decrypt the protected 4 bits of the first byte */
        packet[0] = packet[0] ^ (mask[0] & 0b00001111);
        /* get the packet number length */
        let pn_len = ((packet[0] & 0b00000011) + 1) as usize;
        /* reserved bits (2); always 0b00 */
        if packet[0] & 0b00001100 != 0 { return 0; }
        /* decrypt the packet number bytes */
        let pn_value = match pn_len {
            1 => {
                packet[off] ^= mask[1];
                let pn = &packet[off..off + pn_len];
                pn[0] as usize
            },
            2 => {
                packet[off] ^= mask[1]; packet[off + 1] ^= mask[2];
                let pn = &packet[off..off + pn_len];
                ((pn[0] as usize) << 8) | (pn[1] as usize)
            },
            3 => {
                packet[off] ^= mask[1]; packet[off + 1] ^= mask[2]; packet[off + 2] ^= mask[3];
                let pn = &packet[off..off + pn_len];
                ((pn[0] as usize) << 16) | ((pn[1] as usize) << 8) | (pn[2] as usize)
            },
            4 => {
                packet[off] ^= mask[1]; packet[off + 1] ^= mask[2]; packet[off + 2] ^= mask[3]; packet[off + 3] ^= mask[4];
                let pn = &packet[off..off + pn_len];
                ((pn[0] as usize) << 24) | ((pn[1] as usize) << 16) | ((pn[2] as usize) << 8) | (pn[3] as usize)
            },
            _ => return 0, // unsupported packet number length
        };
        Some(Self {
            flags,
            version,
            destination_connection_id,
            source_connection_id,
            token,
        })
    } */
}