//! Exact packed bit transport; unused bits must be zero.
#[derive(Clone, Copy)]
pub(super) enum Packing {
    Msb,
    Lsb,
}
pub(super) fn packed(data: &[u8], bits: usize, order: Packing) -> bool {
    if data.len() != bits.div_ceil(8) {
        return false;
    }
    let used = bits % 8;
    used == 0
        || data.last().is_some_and(|byte| {
            byte & match order {
                Packing::Msb => u8::MAX >> used,
                Packing::Lsb => u8::MAX << used,
            } == 0
        })
}
