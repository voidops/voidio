use std::sync::Arc;
use ring::aead::{Aad, LessSafeKey, UnboundKey, AES_128_GCM};
use ring::hkdf::{Salt, Prk, HKDF_SHA256};
use rustls::{ClientConfig, RootCertStore};
use rustls::crypto::ring::default_provider;
use rustls::pki_types::ServerName;
use crate::net::{aes_block, encode_varint_into, expand_label, make_nonce};

pub fn build_initial_packet(
    initial_salt: &Salt,
    dcid: &[u8],
    scid: &[u8],
    client_hello: &[u8],
) -> Vec<u8> {
    let pn_len = 2usize;
    let packet_number = 0u32;

    // ===== CRYPTO frame =====
    let mut crypto = Vec::new();
    crypto.push(0x06);
    encode_varint_into(0, &mut crypto);
    encode_varint_into(client_hello.len() as u64, &mut crypto);
    crypto.extend_from_slice(client_hello);

    let tag_len = AES_128_GCM.tag_len();
    let cipher_len = crypto.len() + tag_len;
    let length_field_value = (pn_len + cipher_len) as u64;

    // ===== header =====
    let mut packet = Vec::with_capacity(1200);

    packet.push(0xC0 | (pn_len - 1) as u8);          // long header, fixed bit, initial
    packet.extend_from_slice(&1u32.to_be_bytes());   // version

    packet.push(dcid.len() as u8);
    packet.extend_from_slice(dcid);

    packet.push(scid.len() as u8);
    packet.extend_from_slice(scid);

    packet.push(0); // token length = 0

    encode_varint_into(length_field_value, &mut packet);

    let pn_pos = packet.len();
    let pn_be = (packet_number as u16).to_be_bytes();
    packet.extend_from_slice(&pn_be[..pn_len]);

    let header_len = packet.len();
    let header = &packet[..header_len];

    // ===== key derivation (exact match to your server) =====

    // initial_secret = HKDF-Extract(salt, dcid)
    let initial_secret: Prk = initial_salt.extract(dcid);

    // client_initial = Expand(initial_secret, "client in")
    let mut client_init = [0u8; 32];
    expand_label(&initial_secret, b"client in", &mut client_init);

    // prk = Prk(client_init)
    let prk = Prk::new_less_safe(HKDF_SHA256, &client_init);

    let mut hp_key = [0u8; 16];
    expand_label(&prk, b"quic hp", &mut hp_key);

    let mut aead_key_buf = [0u8; 16];
    expand_label(&prk, b"quic key", &mut aead_key_buf);

    let mut iv = [0u8; 12];
    expand_label(&prk, b"quic iv", &mut iv);

    // ===== AEAD encrypt =====
    let unbound = UnboundKey::new(&AES_128_GCM, &aead_key_buf).unwrap();
    let key = LessSafeKey::new(unbound);
    let nonce = make_nonce(packet_number, &iv);

    let mut payload = crypto;
    key.seal_in_place_append_tag(nonce, Aad::from(header), &mut payload)
        .unwrap();

    packet.extend_from_slice(&payload);

    // ===== header protection =====
    let sample_start = pn_pos + pn_len + 4;
    let sample_end   = sample_start + 16;

    if sample_end <= packet.len() {
        let sample: [u8;16] = packet[sample_start..sample_end].try_into().unwrap();
        let mask = aes_block(&hp_key, &sample);

        packet[0] ^= mask[0] & 0x0F;
        for i in 0..pn_len {
            packet[pn_pos + i] ^= mask[i + 1];
        }
    }

    packet
}
fn encode_quic_transport_parameters(scid: &[u8]) -> Vec<u8> {
    // TLS-encoded TransportParameters (RFC 9000 §18):
    // TransportParameters { parameters<0..2^16-1> }
    //   TransportParameter { parameter (u16), value<0..2^16-1> }
    // We include:
    //   - initial_source_connection_id (0x000f) -> raw CID bytes
    //   - max_udp_payload_size (0x0003)        -> QUIC varint (use 65527)
    //   - active_connection_id_limit (0x000e)  -> QUIC varint (use 4)
    let mut body = Vec::with_capacity(64);

    // initial_source_connection_id = 0x000f
    body.extend_from_slice(&0x000fu16.to_be_bytes());
    body.extend_from_slice(&(scid.len() as u16).to_be_bytes());
    body.extend_from_slice(scid);

    // max_udp_payload_size = 65527
    let mut tmp = Vec::with_capacity(8);
    encode_varint_into(65_527, &mut tmp);
    body.extend_from_slice(&0x0003u16.to_be_bytes());
    body.extend_from_slice(&(tmp.len() as u16).to_be_bytes());
    body.extend_from_slice(&tmp);

    // active_connection_id_limit = 4
    tmp.clear();
    encode_varint_into(4, &mut tmp);
    body.extend_from_slice(&0x000eu16.to_be_bytes());
    body.extend_from_slice(&(tmp.len() as u16).to_be_bytes());
    body.extend_from_slice(&tmp);

    let mut out = Vec::with_capacity(2 + body.len());
    out.extend_from_slice(&(body.len() as u16).to_be_bytes());
    out.extend_from_slice(&body);
    out
}

fn init_crypto() {
    rustls::crypto::CryptoProvider::install_default(default_provider()).unwrap();
}

/*
pub fn build_quic_client_hello(server_name: ServerName<'static>, scid: &[u8], alpn: &[&[u8]]) -> Vec<u8> {
init_crypto();
// Root store (uses system roots from webpki-roots)
let root_store: RootCertStore =
    RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

// rustls client config (default provider, TLS1.3 enabled by default)
let config: ClientConfig = ClientConfig::builder()
    .with_root_certificates(root_store)
    .with_no_client_auth();
let params = encode_quic_transport_parameters(scid);
let alpn_vec: Vec<Vec<u8>> = alpn.iter().map(|p| p.to_vec()).collect();

let mut conn = ClientConnection::new_with_alpn(
    Arc::new(config),
    Version::V1,
    server_name,
    params,
    alpn_vec,
)
    .expect("quic client conn");

let mut client_hello = Vec::with_capacity(1024);
let _ = conn.write_hs(&mut client_hello);
client_hello
}
*/