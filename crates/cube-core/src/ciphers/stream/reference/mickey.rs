//! MICKEY2.0's 100-bit registers; not MICKEY-128 or the original80-bit state.
use zeroize::Zeroizing;
const MASK: u128 = (1u128 << 100) - 1;
const R_MASK: u128 = 0x3_df87818f_b5546660_1279327b;
const COMP0: u128 = 0x6_057ebfea_7942a809_6aa97a30;
const COMP1: u128 = 0x1_91c23dd7_e3a21d63_dd629e9a;
const FB0: u128 = 0x1_9cec5802_af4a9381_9ffa7faf;
const FB1: u128 = 0x8_40fbc52b_4911b063_4c8cb877;
fn clock(state: &mut [u128; 2], mixing: bool, input: u128) -> u128 {
    let [r, s] = *state;
    let out = (r ^ s) & 1;
    let control_r = ((s >> 34) ^ (r >> 67)) & 1;
    let control_s = ((s >> 67) ^ (r >> 33)) & 1;
    let input_r = input ^ if mixing { (s >> 50) & 1 } else { 0 };
    let feedback_r = ((r >> 99) ^ input_r) & 1;
    let feedback_s = ((s >> 99) ^ input) & 1;
    state[0] =
        ((r << 1) ^ if control_r != 0 { r } else { 0 } ^ if feedback_r != 0 { R_MASK } else { 0 })
            & MASK;
    let nonlinear = s ^ COMP0; // complemented neighbors are evaluated before shifting.
    let next = (s << 1) ^ ((nonlinear & ((s >> 1) ^ COMP1)) & ((1u128 << 99) - 2));
    state[1] = (next
        ^ if feedback_s != 0 {
            if control_s != 0 { FB1 } else { FB0 }
        } else {
            0
        })
        & MASK;
    out
}
pub(super) fn apply(output: &mut [u8], key: &[u8], iv: &[u8], iv_bits: usize, data_bits: usize) {
    let mut state = Zeroizing::new([0u128; 2]);
    for i in 0..iv_bits {
        clock(&mut state, true, u128::from((iv[i / 8] >> (7 - i % 8)) & 1));
    }
    for i in 0..80 {
        clock(
            &mut state,
            true,
            u128::from((key[i / 8] >> (7 - i % 8)) & 1),
        );
    }
    for _ in 0..100 {
        clock(&mut state, true, 0);
    }
    for i in 0..data_bits {
        output[i / 8] ^= (clock(&mut state, false, 0) as u8) << (7 - i % 8);
    }
}
