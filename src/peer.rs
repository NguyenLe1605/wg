use crate::handshake::Handshake;
use crate::{
    SessionKey, WIREGUARD_COOKIE_LEN, WIREGUARD_MAX_SRC_IPS, WIREGUARD_SESSION_KEY_LEN,
    WIREGUARD_TAI64N_LEN,
};
use heapless::Vec;
use std::net::{IpAddr, SocketAddr};
use x25519_dalek::{PublicKey, SharedSecret};

pub struct Peer {
    // Is this peer initialised?
    valid: bool,
    // Should we be actively trying to connect?
    active: bool,
    // Configured IP and port of the peer (endpoint)
    ep: Endpoint,
    // keep-alive interval in seconds, 0 is disable
    keepalive_interval: u16,
    allowed_source_ips: Vec<AllowedIp, WIREGUARD_MAX_SRC_IPS>,
    public_key: PublicKey,
    preshared_key: SessionKey,
    // Precomputed DH(Sprivi,Spubr) with device private key, and peer public key
    public_key_dh: SharedSecret,

    // Session Keypair
    curr_keypair: KeyPair,
    prev_keypair: KeyPair,
    next_keypair: KeyPair,

    // 5.1 Silence is a Virtue: The responder keeps track of the greatest timestamp received per peer
    greatest_timestamp: Vec<u8, WIREGUARD_TAI64N_LEN>,
    // The active handshake that is happening
    handshake: Handshake,

    // Decrypted cookie from the responder
    cookie_millis: u32,
    cookie: Vec<u8, WIREGUARD_COOKIE_LEN>,

    // The latest mac1 we sent with initiation
    handshake_mac1_valid: bool,
    handshake_mac1: Vec<u8, WIREGUARD_COOKIE_LEN>,

    // Precomputed keys for use in mac validation
    label_cookie_key: Vec<u8, WIREGUARD_SESSION_KEY_LEN>,
    lable_mac1_key: Vec<u8, WIREGUARD_SESSION_KEY_LEN>,

    // The last time we received a valid initiation message
    last_initiation_rx: u32,
    // The last time we sent an initiation message to this peer
    last_initiation_tx: u32,

    // last_tx and last_rx of data packets
    last_tx: u32,
    last_rx: u32,

    // We set this flag on RX/TX of packets if we think that we should initiate a new handshake
    send_handshake: bool,
}

pub struct KeyPair {
    // TODO: Don't know what is valid and what is not
    valid: bool,
    // Did we initiate this session (send the initiation packet rather than sending the response packet)
    initiator: bool,
    // TODO: millies for timeout?
    keypair_millis: u32,
    sending_key: SessionKey,
    sending_valid: bool,

    receiving_key: SessionKey,
    receiving_valid: bool,

    //TODO: What is for rx, what is for tx???
    last_tx: u32,
    last_rx: u32,

    // TODO: Is this to counter against the replay attack?
    replay_bitmap: u32,
    replay_counter: u64,

    // This is the index we generated for our end
    local_index: u32,
    // This is the index on the other end
    remote_index: u32,
}

pub struct AllowedIp {
    valid: bool,
    ip: IpAddr,
    // TODO: Why we need a mask
    mask: IpAddr,
}

pub struct Endpoint {
    endpoint: Option<SocketAddr>,
}
