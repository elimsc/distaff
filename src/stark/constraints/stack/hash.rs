use super::{are_equal, enforce_stack_copy, field, EvaluationResult, HASH_STATE_WIDTH};
use crate::utils::hasher::{apply_inv_mds, apply_mds, apply_sbox};

/// Evaluates constraints for a single round of a modified Rescue hash function. Hash state is
/// assumed to be in the first 6 registers of user stack; the rest of the stack does not change.
pub fn enforce_rescr(
    result: &mut [u128],
    old_stack: &[u128],
    new_stack: &[u128],
    ark: &[u128],
    op_flag: u128,
) {
    // evaluate the first half of Rescue round
    let mut old_state = [field::ZERO; HASH_STATE_WIDTH];
    old_state.copy_from_slice(&old_stack[..HASH_STATE_WIDTH]);

    apply_sbox(&mut old_state);
    apply_mds(&mut old_state);
    for i in 0..HASH_STATE_WIDTH {
        old_state[i] = field::add(old_state[i], ark[i]);
    }

    // evaluate inverse of the second half of Rescue round
    let mut new_state = [field::ZERO; HASH_STATE_WIDTH];
    new_state.copy_from_slice(&new_stack[..HASH_STATE_WIDTH]);
    apply_inv_mds(&mut new_state);
    apply_sbox(&mut new_state);
    for i in 0..HASH_STATE_WIDTH {
        new_state[i] = field::sub(new_state[i], ark[HASH_STATE_WIDTH + i]);
    }

    // compar the results of both rounds
    for i in 0..HASH_STATE_WIDTH {
        result.agg_constraint(i, op_flag, are_equal(new_state[i], old_state[i]));
    }

    // make sure the rest of the stack didn't change
    enforce_stack_copy(result, old_stack, new_stack, HASH_STATE_WIDTH, op_flag);
}

/// Evaluates constraints for a single round of a modified Rescue cipher function.
/// key: 0..4, data: 4..8
/// apply_round(data_state, key_state):
///     key_state = round_step1(key_state, ark)
///     data_state = round_step2(data_state, key_state)
///     key_state = round_step2(key_state, ark)
///     data_state = round_step2(data_state, key_state)
pub fn enforce_rescr_cipher(
    result: &mut [u128],
    old_stack: &[u128],
    new_stack: &[u128],
    ark: &[u128],
    op_flag: u128,
) {
    // evaluate the first half of Rescue round
    let mut old_key_state = [0 as u128; HASH_STATE_WIDTH];
    let mut old_data_state = [0 as u128; HASH_STATE_WIDTH];
    old_key_state.copy_from_slice(&old_stack[..HASH_STATE_WIDTH]);
    old_data_state.copy_from_slice(&old_stack[HASH_STATE_WIDTH..HASH_STATE_WIDTH * 2]);

    apply_sbox(&mut old_key_state);
    apply_mds(&mut old_key_state);
    for i in 0..HASH_STATE_WIDTH {
        // old_key_state[i] += ark[i];
        old_key_state[i] = field::add(old_key_state[i], ark[i]);
    }

    apply_sbox(&mut old_data_state);
    apply_mds(&mut old_data_state);
    for i in 0..HASH_STATE_WIDTH {
        // old_data_state[i] += old_key_state[i];
        old_data_state[i] = field::add(old_data_state[i], old_key_state[i]);
    }

    // evaluate inverse of the second half of Rescue round
    let mut new_key_state = [0 as u128; HASH_STATE_WIDTH];
    let mut new_data_state = [0 as u128; HASH_STATE_WIDTH];
    new_key_state.copy_from_slice(&new_stack[..HASH_STATE_WIDTH]);
    new_data_state.copy_from_slice(&new_stack[HASH_STATE_WIDTH..HASH_STATE_WIDTH * 2]);

    for i in 0..HASH_STATE_WIDTH {
        // new_data_state[i] -= new_key_state[i];
        new_data_state[i] = field::sub(new_data_state[i], new_key_state[i]);
    }
    apply_inv_mds(&mut new_data_state);
    apply_sbox(&mut new_data_state);

    for i in 0..HASH_STATE_WIDTH {
        // new_key_state[i] -= ark[HASH_STATE_WIDTH + i];
        new_key_state[i] = field::sub(new_key_state[i], ark[HASH_STATE_WIDTH + i]);
    }
    apply_inv_mds(&mut new_key_state);
    apply_sbox(&mut new_key_state);

    // compare the results of both rounds
    for i in 0..HASH_STATE_WIDTH {
        result.agg_constraint(i, op_flag, are_equal(new_key_state[i], old_key_state[i]));
    }
    for i in 0..HASH_STATE_WIDTH {
        result.agg_constraint(i, op_flag, are_equal(new_data_state[i], old_data_state[i]));
    }

    // make sure the rest of the stack didn't change
    enforce_stack_copy(result, old_stack, new_stack, HASH_STATE_WIDTH * 2, op_flag);
}
