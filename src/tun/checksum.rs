fn checksum_no_fold(pkt: &[u8], initial: u64) -> u64 {
    let mut ac = initial;
    let mut i = 0;
    let mut n = pkt.len();
    while n >= 4 {
        ac += u32::from_be_bytes([pkt[i], pkt[i + 1], pkt[i + 2], pkt[i + 3]]) as u64;
        i += 4;
        n -= 4;
    }

    while n >= 2 {
        ac += u16::from_be_bytes([pkt[i], pkt[i + 1]]) as u64;
        i += 2;
        n -= 2;
    }

    if n == 1 {
        ac += (pkt[i] as u64) << 8;
    }

    ac
}

pub fn checksum(pkt: &[u8], initial: u64) -> u16 {
    let mut ac = checksum_no_fold(pkt, initial);
    ac = (ac >> 16) + (ac & 0xffff);
    ac = (ac >> 16) + (ac & 0xffff);
    ac = (ac >> 16) + (ac & 0xffff);
    ac = (ac >> 16) + (ac & 0xffff);
    ac as u16
}

pub fn pseudo_header_checksum_no_fold(
    protocol: u8,
    src_addr: &[u8],
    dst_addr: &[u8],
    total_len: u16,
) -> u64 {
    let sum = checksum_no_fold(src_addr, 0);
    let sum = checksum_no_fold(dst_addr, sum);
    let sum = checksum_no_fold(&[0, protocol], sum);
    checksum_no_fold(&total_len.to_be_bytes(), sum)
}
