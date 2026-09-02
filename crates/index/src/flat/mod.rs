use defs::Magic;

pub mod index;
mod serialize;

#[cfg(test)]
mod tests;

pub const FLAT_MAGIC_BYTES: Magic = [0x00, 0x00, 0x00, 0x01];
