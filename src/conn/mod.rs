// Package conn implements WireGuard's network connections.
pub(crate) const IDEAL_BATCH_SIZE: usize = 128; // maximum number of packets handled per read and write
