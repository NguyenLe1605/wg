use blake2::{Blake2s256, Digest};
// use chacha20poly1305::{self, aead::OsRng, ChaCha20Poly1305, KeyInit};
// use heapless::Vec;
// use parking_lot::{Mutex, MutexGuard};
// use socket2::{Domain, Protocol, Socket, Type};
// use std::io;
// use std::net::{IpAddr, SocketAddr};
// use tai64::Tai64N;

mod device;
mod handshake;
mod message;
mod peer;
pub use handshake::Handshake;
pub use peer::Peer;

pub type SessionKey = chacha20poly1305::Key;

// tai64n contains 64-bit seconds and 32-bit nano offset (12 bytes)
const WIREGUARD_TAI64N_LEN: usize = 12;
// Auth algorithm is chacha20pol1305 which is 128bit (16 byte) authenticator
const WIREGUARD_AUTHTAG_LEN: usize = 16;
// Hash algorithm is blake2s which makes 32 byte hashes
const WIREGUARD_HASH_LEN: usize = 32;
// Public key algo is curve22519 which uses 32 byte keys
const WIREGUARD_PUBLIC_KEY_LEN: usize = 32;
// Public key algo is curve22519 which uses 32 byte keys
const WIREGUARD_PRIVATE_KEY_LEN: usize = 32;
// Symmetric session keys are chacha20/poly1305 which uses 32 byte keys
const WIREGUARD_SESSION_KEY_LEN: usize = 32;

// Timers / Limits
const WIREGUARD_COOKIE_LEN: usize = 16;
const COOKIE_SECRET_MAX_AGE: usize = 2 * 60;
const COOKIE_NONCE_LEN: usize = 24;

const REKEY_AFTER_MESSAGES: usize = (1u64 << 60) as usize;
const REJECT_AFTER_MESSAGES: usize = (0xFFFFFFFFFFFFFFFF - (1u64 << 13)) as usize;
const REKEY_AFTER_TIME: usize = 120;
const REJECT_AFTER_TIME: usize = 180;
const REKEY_TIMEOUT: usize = 5;
const KEEPALIVE_TIMEOUT: usize = 10;

// Peers are allocated statically inside the device structure to avoid malloc
const WIREGUARD_MAX_PEERS: usize = 1;
const WIREGUARD_MAX_SRC_IPS: usize = 2;
// Per device limit on accepting (valid) initiation requests - per peer
const MAX_INITIATIONS_PER_SECOND: usize = 2;

// 5.4 Messages
// Constants
// The UTF-8 string literal "Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s", 37 bytes of output
const CONSTRUCTION: &[u8; 37] = b"Noise_IKpsk2_25519_ChaChaPoly_BLAKE2s";
// The UTF-8 string literal "WireGuard v1 zx2c4 Jason@zx2c4.com", 34 bytes of output
const IDENTIFIER: &[u8; 34] = b"WireGuard v1 zx2c4 Jason@zx2c4.com";
// Label-Mac1 The UTF-8 string literal "mac1----", 8 bytes of output.
const LABEL_MAC1: &[u8; 8] = b"mac1----";
// Label-Cookie The UTF-8 string literal "cookie--", 8 bytes of output
const LABEL_COOKIE: &[u8; 8] = b"cookie--";
const BASE64_LOOKUP: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const ZERO_KEY: &[u8] = &[0u8; WIREGUARD_PUBLIC_KEY_LEN];

pub struct WireguardCtx {
    // TODO: Why these construction hash
    construction_hash: [u8; WIREGUARD_HASH_LEN],
    identifier_hash: [u8; WIREGUARD_HASH_LEN],
}

impl WireguardCtx {
    pub fn init() -> Self {
        let construction_hash = Blake2s256::new()
            .chain_update(CONSTRUCTION)
            .finalize()
            .into();
        let identifier_hash = Blake2s256::new().chain_update(IDENTIFIER).finalize().into();
        Self {
            construction_hash,
            identifier_hash,
        }
    }
}

// pub struct Peer {
//     endpoint: Mutex<Option<SocketAddrV4>>,
// }
//
// pub struct Device {
//     udp: UdpSocket,
//     iface: Iface,
//     peer: Peer,
// }
//
// impl Device {
//     pub fn new(iface: Iface, peer: Option<SocketAddrV4>) -> Self {
//         let session_key: SessionKey = ChaCha20Poly1305::generate_key(&mut OsRng);
//         let udp = new_udp_socket(19988).unwrap();
//         Self {
//             udp,
//             iface,
//             peer: Peer {
//                 endpoint: Mutex::new(peer),
//             },
//         }
//     }
//
//     pub fn loop_listen_iface(&self) -> io::Result<()> {
//         let mut buf = [0u8; 1504];
//         {
//             let peer = self.peer.endpoint();
//             if let Some(peer_addr) = peer.as_ref() {
//                 eprintln!("initiating \"handshake\" to peer: {peer_addr}");
//                 self.udp.send_to("hello?".as_bytes(), peer_addr)?;
//             }
//         }
//
//         loop {
//             let nbytes = self.iface.recv(&mut buf[..])?;
//             match etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]) {
//                 Ok(iph) => {
//                     let src = iph.source_addr();
//                     let dst = iph.destination_addr();
//                     eprintln!("Got Ipv4 packet of size: {nbytes}, {src} -> {dst}, from tun0");
//                 }
//                 _ => {}
//             }
//             let peer = self.peer.endpoint();
//             if let Some(peer_addr) = peer.as_ref() {
//                 self.udp.send_to(&buf[..nbytes], peer_addr)?;
//             } else {
//                 eprintln!("..no peer");
//             }
//         }
//     }
//
//     pub fn loop_listen_udp(&self) -> io::Result<()> {
//         let mut buf = [0u8; 1504];
//
//         loop {
//             let (nbytes, peer_addr) = self.udp.recv_from(&mut buf[..])?;
//             eprintln!("Got packet of size: {nbytes}, from {peer_addr}");
//
//             match etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes]) {
//                 Ok(iph) => {
//                     let src = iph.source_addr();
//                     let dst = iph.destination_addr();
//                     eprintln!("  {src} -> {dst}");
//                 }
//                 _ => {
//                     eprintln!("not an Ipv4 packet");
//                 }
//             }
//
//             if let SocketAddr::V4(peer_addr_v4) = peer_addr {
//                 if &buf[..nbytes] == b"hello?" {
//                     self.peer.set_endpoint(peer_addr_v4);
//                     continue;
//                 }
//                 self.iface.send(&buf[..nbytes])?;
//             }
//         }
//     }
// }
//
// fn new_udp_socket(port: u16) -> io::Result<UdpSocket> {
//     let socket_addr = SocketAddr::from(([0, 0, 0, 0], port));
//
//     let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
//
//     socket.set_reuse_address(true)?;
//
//     socket.bind(&socket_addr.into())?;
//
//     Ok(socket.into())
// }
//
// impl Peer {
//     fn endpoint(&self) -> MutexGuard<Option<SocketAddrV4>> {
//         self.endpoint.lock()
//     }
//
//     fn set_endpoint(&self, addr: SocketAddrV4) {
//         let mut endpoint = self.endpoint.lock();
//         if endpoint.is_none() {
//             *endpoint = Some(addr);
//         }
//     }
// }
