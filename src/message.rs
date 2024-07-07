use crate::{COOKIE_NONCE_LEN, WIREGUARD_AUTHTAG_LEN, WIREGUARD_COOKIE_LEN, WIREGUARD_TAI64N_LEN};

pub(crate) enum Message<'a> {
    HandshakeInitation(HandshakeInitiation<'a>),
    HandshakeResponse(HandshakeResponse<'a>),
    CookieReply(CookieReply<'a>),
    TransportData(TransportData<'a>),
}

// 5.4.2 First Message: Initiator to Responder
pub(crate) struct HandshakeInitiation<'a> {
    sender: u32,
    ephemeral: &'a [u8; 32],
    enc_static: &'a [u8; 32 + WIREGUARD_AUTHTAG_LEN],
    enc_timestamp: &'a [u8; WIREGUARD_TAI64N_LEN + WIREGUARD_AUTHTAG_LEN],
    mac1: &'a [u8; WIREGUARD_COOKIE_LEN],
    mac2: &'a [u8; WIREGUARD_COOKIE_LEN],
}

// 5.4.3 Second Message: Responder to Initiator
pub(crate) struct HandshakeResponse<'a> {
    sender: u32,
    receiver: u32,
    ephemeral: &'a [u8; 32],
    mac1: &'a [u8; WIREGUARD_COOKIE_LEN],
    mac2: &'a [u8; WIREGUARD_COOKIE_LEN],
}

// 5.4.7 Under Load: Cookie Reply Message
pub(crate) struct CookieReply<'a> {
    receiver: u32,
    nonce: &'a [u8; COOKIE_NONCE_LEN],
    enc_cookie: &'a [u8; WIREGUARD_COOKIE_LEN + WIREGUARD_AUTHTAG_LEN],
}

// 5.4.6 Subsequent Messages: Transport Data Messages
pub(crate) struct TransportData<'a> {
    receiever: u32,
    counter: &'a [u8; 8],
    // Followed by encrypted data
    enc_packet: &'a [u8],
}
