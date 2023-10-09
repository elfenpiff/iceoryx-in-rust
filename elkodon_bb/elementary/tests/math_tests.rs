use elkodon_bb_elementary::math::*;
use elkodon_bb_testing::assert_that;

#[test]
fn math_align_returns_input_when_already_aligned() {
    assert_that!(align(25, 5), eq 25);
}

#[test]
fn math_align_returns_input_to_next_greater_value() {
    assert_that!(align(30, 7), eq 35);
}

#[test]
fn math_log2_of_power_of_2_works() {
    assert_that!(0, eq log2_of_power_of_2(0));
    for i in 0..64 {
        assert_that!(i as u8, eq log2_of_power_of_2(2u64.pow(i)));
    }
}

#[test]
fn math_round_to_pow2_works() {
    assert_that!(round_to_pow2(1), eq 1);
    assert_that!(round_to_pow2(2), eq 2);
    assert_that!(round_to_pow2(3), eq 4);
    assert_that!(round_to_pow2(4), eq 4);
    assert_that!(round_to_pow2(5), eq 8);
    assert_that!(round_to_pow2(6), eq 8);
    assert_that!(round_to_pow2(8589934597), eq 17179869184);
}
