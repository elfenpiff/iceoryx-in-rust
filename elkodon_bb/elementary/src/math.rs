//! Contains simplistic math functions.

/// Aligns value to alignment. It increments value to the next multiple of alignment.
pub const fn align(value: usize, alignment: usize) -> usize {
    if value % alignment == 0 {
        value
    } else {
        value + alignment - value % alignment
    }
}

/// Aligns value to the alignment of T.
pub const fn align_to<T>(value: usize) -> usize {
    align(value, std::mem::align_of::<T>())
}

/// Calculates log2 of a number which is a power of 2
pub fn log2_of_power_of_2(value: u64) -> u8 {
    let mut bits = value;

    for i in 0..64 {
        if bits == 1 {
            return i;
        }

        bits >>= 1;
    }

    0
}

pub fn round_to_pow2(mut value: u64) -> u64 {
    value -= 1;
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    value |= value >> 32;
    value += 1;

    value
}
