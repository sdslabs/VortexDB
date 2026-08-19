use defs::Magic;

pub const KD_TREE_MAGIC_BYTES: Magic = [0x00, 0x01, 0x02, 0x00];

pub(super) const NODE_MARKER_BYTE: u8 = 1u8;
pub(super) const SKIP_MARKER_BYTE: u8 = 0u8;

pub(super) const DELETED_MASK: u8 = 2u8;
