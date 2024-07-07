use crate::WIREGUARD_HASH_LEN;
use heapless::Vec;
use x25519_dalek::{EphemeralSecret, PublicKey};

pub struct Handshake {
    valid: bool,
    initiator: bool,
    local_index: u32,
    remote_index: u32,
    ephemeral_private: EphemeralSecret,
    remote_pubkey: PublicKey,
    hash: Vec<u8, WIREGUARD_HASH_LEN>,
    chaining_key: Vec<u8, WIREGUARD_HASH_LEN>,
}
