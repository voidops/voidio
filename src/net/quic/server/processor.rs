use std::net::SocketAddr;
use ring::hkdf::{Prk, HKDF_SHA256};
use crate::net::{aes_block, expand_label, varint};
use crate::net::connection::{ConnectionId, QuicConnection, QuicConnectionType};
use super::QuicThreadContext;

/*
Initial Packet {
  Header Form (1) = 1,
  Fixed Bit (1) = 1,
  Long Packet Type (2) = 0,
  Reserved Bits (2),         # Protected
  Packet Number Length (2),  # Protected
  Version (32),
  DCID Len (8),
  Destination Connection ID (0..160),
  SCID Len (8),
  Source Connection ID (0..160),
  Token Length (i),
  Token (..),
  Length (i),
  Packet Number (8..32),     # Protected
  Protected Payload (0..24), # Skipped Part
  Protected Payload (128),   # Sampled Part
  Protected Payload (..)     # Remainder
}
*/

#[cfg_attr(not(debug_assertions), inline(always))]
fn update_connection<'ctx>(
    ctx: &'ctx mut QuicThreadContext,
    scid: ConnectionId,
    dcid: ConnectionId,
    packet_number: u32,
    source_address: &SocketAddr,
) -> &'ctx mut QuicConnection {
    use std::collections::hash_map::Entry;
    match ctx.connections.entry(scid) {
        Entry::Occupied(entry) => {
            let conn = entry.into_mut();
            conn.last_packet_number = packet_number;
            conn.address = *source_address;
            conn
        }
        Entry::Vacant(entry) => {
            entry.insert(QuicConnection::new(
                scid,
                dcid,
                packet_number,
                source_address,
                QuicConnectionType::Server,
            ))
        }
    }
}

#[cfg_attr(not(debug_assertions), inline(always))]
pub(crate) fn exec_quic_initial(ctx: &mut QuicThreadContext, packet: &mut [u8], source_address: &SocketAddr) -> usize {
    let dcid_len = packet[5] as usize;
    let dcid_start = 6;
    let dcid_end = dcid_start + dcid_len;

    if dcid_len > 20 || dcid_end >= packet.len() { return 0; }

    let dcid = ConnectionId::from_slice(&packet[6..dcid_end]);

    let scid_len = packet[dcid_end] as usize;
    let scid_start = dcid_end + 1;
    let scid_end = scid_start + scid_len;

    if scid_len > 20 || scid_end > packet.len() { return 0; }

    let scid = ConnectionId::from_slice(&packet[scid_start .. scid_end]);

    /* token length */
    let Some((token_len, tlen_len)) = varint(&packet[scid_end..]) else { return 0; };
    let token_start = scid_end + tlen_len;
    let token_end = token_start + token_len;
    if token_end > packet.len() { return 0; }
    let token_slice = &packet[token_start..token_end];

    /* derive secrets */
    let initial_secret = ctx.initial_salt.extract(dcid.as_bytes());
    expand_label(&initial_secret, b"client in", &mut ctx.client_init_buf);
    let prk = Prk::new_less_safe(HKDF_SHA256, &ctx.client_init_buf);
    expand_label(&prk, b"quic hp", &mut ctx.hp_key_buf);

    /* locate packet-number offset */
    let mut off = token_end;
    let Some((packet_length, pl)) = varint(&packet[off..]) else { return 0; };
    off += pl; // PN offset
    if off + 20 > packet.len() { return 0; }
    let sample: &[u8;16] = (&packet[off+4..off+20]).try_into().unwrap();
    let mask = aes_block(&ctx.hp_key_buf, sample);

    /* decrypt the protected 4 bits of the first byte */
    packet[0] = packet[0] ^ (mask[0] & 0b00001111);

    /* get the packet number length */
    let pn_len = ((packet[0] & 0b00000011) + 1) as usize;

    /* reserved bits (2); always 0b00 */
    if packet[0] & 0b00001100 != 0 { return 0; }

    /* decrypt the packet number bytes */
    let packet_number = match pn_len {
        1 => {
            packet[off] ^= mask[1];
            let pn = &packet[off..off + pn_len];
            pn[0] as u32
        },
        2 => {
            packet[off] ^= mask[1]; packet[off + 1] ^= mask[2];
            let pn = &packet[off..off + pn_len];
            ((pn[0] as u32) << 8) | (pn[1] as u32)
        },
        3 => {
            packet[off] ^= mask[1]; packet[off + 1] ^= mask[2]; packet[off + 2] ^= mask[3];
            let pn = &packet[off..off + pn_len];
            ((pn[0] as u32) << 16) | ((pn[1] as u32) << 8) | (pn[2] as u32)
        },
        4 => {
            packet[off] ^= mask[1]; packet[off + 1] ^= mask[2]; packet[off + 2] ^= mask[3]; packet[off + 3] ^= mask[4];
            let pn = &packet[off..off + pn_len];
            ((pn[0] as u32) << 24) | ((pn[1] as u32) << 16) | ((pn[2] as u32) << 8) | (pn[3] as u32)
        },
        _ => return 0, // unsupported packet number length
    };

    let payload_start  = off + pn_len;
    let quic_end = off + packet_length;

    if quic_end > packet.len() { return 0; } // invalid packet length

    /* derive aead key & iv */
    expand_label(&prk, b"quic key", &mut ctx.aead_key_buf);
    expand_label(&prk, b"quic iv",  &mut ctx.aead_iv_buf);

    use ring::aead::{self, LessSafeKey, Nonce, UnboundKey, Aad};

    let aead_key = LessSafeKey::new(UnboundKey::new(&aead::AES_128_GCM, &ctx.aead_key_buf).unwrap());

    /* build nonce = iv XOR padded packet_number (big-endian, 8 bytes) */

    let mut nonce_bytes = [0u8; 12];
    nonce_bytes.copy_from_slice(&ctx.aead_iv_buf);
    let pn_be = (packet_number as u64).to_be_bytes();
    for i in 0..8 { nonce_bytes[nonce_bytes.len() - 8 + i] ^= pn_be[i]; }

    /* decrypt in place */
    let (header, rest) = packet.split_at_mut(payload_start);
    let aad = Aad::from(header); // header + pn
    let cipher_slice = &mut rest[..(quic_end - payload_start)];
    aead_key.open_in_place(Nonce::assume_unique_for_key(nonce_bytes), aad, cipher_slice).unwrap();

    /* parse CRYPTO frames here. */

    let mut off = 0;

    /* update connection */

    // does a connection really form here in the initial packet? maybe we must reply first with a ServerHello (Sever's own Initial Packet), and wait for the Client to respond with another Initial Packet containing the Server's DCID as SCID, before we can consider this a valid connection.
    //let conn = update_connection(ctx, scid, dcid, packet_number, source_address);
    // debugging output, always on for now
    if true {
        //println!("[QUIC] {} => Server: Initial, DCID={}, SCID={}, PKN={}, ClientHello", source_address, dcid, scid, packet_number);
        //println!("[QUIC] <Server> {} => {}@{} (#{}): Initial Packet", conn.dcid, conn.id, source_address, conn.last_packet_number);
    }
    /*
    println!("[QUIC] <Client> {}@{} => <Server> {} (#{}): ClientHello (Initial - 1 Crypto Frame)", conn.id, conn.address, conn.dcid, conn.last_packet_number);
    println!("[QUIC] <Server> {} => <Client> {}@{} (#{}): ServerHello (Handshake - 1 Crypto Frame)", conn.dcid, conn.id, source_address, conn.last_packet_number);
    */
    quic_end
}

#[cfg_attr(not(debug_assertions), inline(always))]
pub(crate) fn exec_quic_hanshake(ctx: &mut QuicThreadContext, packet: &mut [u8], source: &SocketAddr) -> usize {
    0
}

#[cfg_attr(not(debug_assertions), inline(always))]
pub(crate) fn exec_quic_retry(ctx: &mut QuicThreadContext, packet: &mut [u8], source: &SocketAddr) -> usize {
    0
}

#[cfg_attr(not(debug_assertions), inline(always))]
pub(crate) fn exec_quic_1rtt(ctx: &mut QuicThreadContext, packet: &mut [u8], source: &SocketAddr) -> usize {
    0
}

#[cfg_attr(not(debug_assertions), inline(always))]
pub(crate) fn exec_quic_version_negotiation(ctx: &mut QuicThreadContext, packet: &mut [u8], source: &SocketAddr) -> usize {
    0
}
fn format_as_vec_literal(bytes: &[u8]) -> String {
    let mut out = String::from("vec![");
    for (i, b) in bytes.iter().enumerate() {
        if i != 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("0x{:02X}", b));
    }
    out.push(']');
    out
}

/* Process a QUIC Packet and returns its size. */
#[cfg_attr(not(debug_assertions), inline(always))]
pub(crate) fn exec_quic_packet(ctx: &mut QuicThreadContext, packet: &mut [u8], source_address: &SocketAddr) -> usize {
    //println!("{}", format_as_vec_literal(packet));
    if packet[0] & 0b10000000 == 0 { /* short header */
        0
    } else { /* long header */

        let quic_version = u32::from_be_bytes(packet[1..5].try_into().unwrap());
        if quic_version != 1 { return 0; }
        if packet[0] & 0b01000000 == 0 { return exec_quic_version_negotiation(ctx, packet, &source_address); }
        let packet_type = (packet[0] & 0b00110000) >> 4;
        match packet_type {
            0b00 => exec_quic_initial(ctx, packet, &source_address),
            0b01 => exec_quic_hanshake(ctx, packet, &source_address),
            0b10 => exec_quic_retry(ctx, packet, &source_address),
            0b11 => exec_quic_1rtt(ctx, packet, &source_address),
            _ => 0,
        }
    }
}