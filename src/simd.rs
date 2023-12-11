#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
use std::arch::aarch64::*;


#[inline]
#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
pub(crate) unsafe fn _mm_movemask_epi8(input: uint8x16_t) -> i32 {
    // Example input (half scale):
    // 0x89 FF 1D C0 00 10 99 33
    // Shift out everything but the sign bits
    // 0x01 01 00 01 00 00 01 00
    let high_bits = vreinterpretq_u16_u8(vshrq_n_u8::<7>(input));
    // Merge the even lanes together with vsra. The '??' bytes are garbage.
    // vsri could also be used, but it is slightly slower on aarch64.
    // 0x??03 ??02 ??00 ??01
    let paired16 = vreinterpretq_u32_u16(vsraq_n_u16::<7>(high_bits, high_bits));
    // Repeat with wider lanes.
    // 0x??????0B ??????04
    let paired32 = vreinterpretq_u64_u32(vsraq_n_u32::<14>(paired16, paired16));
    // 0x??????????????4B
    let paired64 = vreinterpretq_u8_u64(vsraq_n_u64::<28>(paired32, paired32));
    // Extract the low 8 bits from each lane and join.
    // 0x4B
    vgetq_lane_u8::<0>(paired64) as i32 | (vgetq_lane_u8::<8>(paired64) as i32) << 8
}
