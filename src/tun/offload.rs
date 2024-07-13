use super::TunError;
use crate::conn;
use byteorder::{ByteOrder, NetworkEndian};
use std::collections::HashMap;
// implementation of offloading UDP and TCP packet

const TCP_FLAG_OFFSET: usize = 13;
// VIRTIO_NET_HDR_LEN is the length in bytes of VirtioNetHdr. This matches the
// shape of the C ABI for its kernel counterpart -- sizeof(virtio_net_hdr).
pub(crate) const VIRTIO_NET_HDR_LEN: usize = std::mem::size_of::<VirtioNetHdr>();

#[repr(u8)]
#[derive(Debug)]
enum TcpFlag {
    Fin = 0x01,
    Psh = 0x08,
    Ack = 0x10,
}

// defined in linux kernel at include/uapi/linux/virtio_net.h
// show up first in the scatter-gather lists
#[repr(C)]
struct VirtioNetHdr {
    flags: u8,
    gso_type: u8,
    // Ethernet + IP + tcp/udp hdrs
    hdr_len: u16,
    // Bytes to append to hdr_len per frame
    gso_size: u16,
    // Position to start checksumming from
    csum_start: u16,
    // Offset after that to place checksum
    csum_offset: u16,
}

impl VirtioNetHdr {
    fn decode(buf: &[u8]) -> Result<Self, TunError> {
        if buf.len() < VIRTIO_NET_HDR_LEN {
            return Err(TunError::ShortBuffer);
        }
        let flags = buf[0];
        let gso_type = buf[1];
        let hdr_len = NetworkEndian::read_u16(&buf[2..]);
        let gso_size = NetworkEndian::read_u16(&buf[4..]);
        let csum_start = NetworkEndian::read_u16(&buf[6..]);
        let csum_offset = NetworkEndian::read_u16(&buf[8..]);
        Ok(Self {
            flags,
            gso_type,
            hdr_len,
            gso_size,
            csum_start,
            csum_offset,
        })
    }

    fn encode(&self, buf: &mut [u8]) -> Result<(), TunError> {
        if buf.len() < VIRTIO_NET_HDR_LEN {
            return Err(TunError::ShortBuffer);
        }

        buf[0] = self.flags;
        buf[1] = self.gso_type;
        NetworkEndian::write_u16(&mut buf[2..], self.hdr_len);
        NetworkEndian::write_u16(&mut buf[4..], self.gso_size);
        NetworkEndian::write_u16(&mut buf[6..], self.csum_start);
        NetworkEndian::write_u16(&mut buf[8..], self.csum_offset);

        Ok(())
    }
}

// TcpFlowKey represents the key for a TCP flow.
#[derive(Debug, Clone)]
struct TcpFlowKey {
    src_addr: [u8; 16],
    dst_addr: [u8; 16],
    src_port: u16,
    dst_port: u16,
    // varying ack values should not be coalesced. Treat them as separate flows.
    rx_ack: u32,
    is_v6: bool,
}

impl TcpFlowKey {
    pub fn new(
        pkt: &[u8],
        src_addr_offset: usize,
        dst_addr_offset: usize,
        tcph_offset: usize,
    ) -> Self {
        let addr_size = dst_addr_offset - src_addr_offset;
        let mut src_addr = [0u8; 16];
        let mut dst_addr = [0u8; 16];
        src_addr.copy_from_slice(&pkt[src_addr_offset..dst_addr_offset]);
        dst_addr.copy_from_slice(&pkt[dst_addr_offset..(dst_addr_offset + addr_size)]);
        let src_port = NetworkEndian::read_u16(&pkt[tcph_offset..]);
        let dst_port = NetworkEndian::read_u16(&pkt[(tcph_offset + 2)..]);
        let rx_ack = NetworkEndian::read_u32(&pkt[(tcph_offset + 8)..]);
        let is_v6 = addr_size == 16;
        Self {
            src_addr,
            dst_addr,
            src_port,
            dst_port,
            rx_ack,
            is_v6,
        }
    }
}

// TcpGROItem represents bookkeeping data for a TCP packet during the lifetime
// of a GRO evaluation across a vector of packets.
#[derive(Debug)]
struct TcpGROItem {
    key: TcpFlowKey,
    // the sequence number
    sent_seq: u32,
    // the index into the original bufs slice
    buf_index: u16,
    // the number of packets merged into this item
    num_merged: u16,
    // payload size
    gso_size: u16,
    // ip header len
    iph_len: u8,
    // tcp header len
    tcph_len: u8,
    // psh flag is set
    psh_set: bool,
}

// TcpGROTable holds flow and coalescing information for the purposes of TCP GRO.
struct TcpGROTable {
    items_by_flow: HashMap<TcpFlowKey, Vec<TcpGROItem>>,
    items_pool: Vec<Vec<TcpGROItem>>,
}

impl TcpGROTable {
    pub fn new() -> Self {
        let items_by_flow = HashMap::with_capacity(conn::IDEAL_BATCH_SIZE);
        let mut items_pool = Vec::with_capacity(conn::IDEAL_BATCH_SIZE);
        let cap = items_pool.capacity();
        for _ in 0..cap {
            items_pool.push(Vec::with_capacity(conn::IDEAL_BATCH_SIZE));
        }
        Self {
            items_by_flow,
            items_pool,
        }
    }
}
