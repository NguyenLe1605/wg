use crate::{WIREGUARD_HASH_LEN, WIREGUARD_MAX_PEERS, WIREGUARD_SESSION_KEY_LEN};
use heapless::Vec;
// use parking_lot::{Mutex, MutexGuard};
// use socket2::{Domain, Protocol, Socket, Type};
use std::net::UdpSocket;
use tun_tap::Iface;
use x25519_dalek::{PublicKey, StaticSecret};

pub struct Device {
    iface: Iface,
    udp: UdpSocket,

    public_key: PublicKey,
    private_key: StaticSecret,

    cookie_secret: Vec<u8, WIREGUARD_HASH_LEN>,
    cookie_secret_millis: u32,

    // Precalculated
    label_cookie_key: Vec<u8, WIREGUARD_SESSION_KEY_LEN>,
    label_mac1_key: Vec<u8, WIREGUARD_SESSION_KEY_LEN>,

    // List of peers associated with this device
    peers: Vec<Peer, WIREGUARD_MAX_PEERS>,
    valid: bool,
}
