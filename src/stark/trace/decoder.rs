use std::ops::Range;
use crate::processor::opcodes;
use crate::math::{ F64, FiniteField, FieldElement };
use crate::stark::utils::{ AccumulatorBuilder };
use crate::utils::filled_vector;
use super::{ NUM_OP_BITS };

// CONSTANTS
// ================================================================================================
pub const NUM_REGISTERS: usize = 2 + NUM_OP_BITS + F64::ACC_STATE_WIDTH;

pub const OP_CODE_IDX     : usize = 0;
pub const PUSH_FLAG_IDX   : usize = 1;
pub const OP_BITS_RANGE   : Range<usize> = Range { start: 2, end:  7 };
pub const OP_ACC_RANGE    : Range<usize> = Range { start: 7, end: 19 };

pub const PROG_HASH_RANGE : Range<usize> = Range { start: 7, end: 11 };

// TRACE BUILDER
// ================================================================================================

/// Builds decoder execution trace; the trace consists of the following registers:
/// op_code: 1 register
/// push_flag: 1 register
/// op_bits: 5 registers
/// op_acc: 12 registers
pub fn process<T>(program: &[T], extension_factor: usize) -> Vec<Vec<T>>
    where T: FieldElement + FiniteField<T> + AccumulatorBuilder<T>
{

    let trace_length = program.len();
    let domain_size = trace_length * extension_factor;

    assert!(trace_length.is_power_of_two(), "trace length must be a power of 2");
    assert!(extension_factor.is_power_of_two(), "trace extension factor must be a power of 2");
    assert!(program[trace_length - 1] == T::from(opcodes::NOOP), "last operation of a program must be NOOP");

    // create op_code register and copy program into it
    let mut op_code = filled_vector(trace_length, domain_size, T::ZERO);
    op_code.copy_from_slice(program);

    // initialize push_flags and op_bits registers
    let mut push_flag = filled_vector(trace_length, domain_size, T::ZERO);
    let mut op_bits = vec![
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
    ];

    // populate push_flags and op_bits registers
    let mut i = 0;
    while i < trace_length {
        set_op_bits(&mut op_bits, op_code[i].as_u8(), i);
        push_flag[i] = T::ZERO;

        // if the current operation is PUSH, the next operation is a constant to be pushed onto
        // the stack; so, set the push_flag for the next operation to 1 and op_bits to NOOP
        if op_code[i] == T::from(opcodes::PUSH) {
            i += 1;
            set_op_bits(&mut op_bits, opcodes::NOOP, i);
            push_flag[i] = T::ONE;
        }

        i += 1;
    }

    // create op_acc register traces
    let op_acc = hash_program(&op_code, domain_size);

    // move all registers into a single vector
    let mut registers = vec![op_code, push_flag];
    for register in op_bits.into_iter() { registers.push(register); }
    for register in op_acc.into_iter() { registers.push(register); }

    assert!(registers.len() == NUM_REGISTERS, "inconsistent number of decoder registers");
    return registers;
}

// HELPER FUNCTIONS
// ================================================================================================

/// Uses a modified version of Rescue hash function to reduce all op_codes into a single hash value
fn hash_program<T>(op_codes: &[T], domain_size: usize) -> Vec<Vec<T>>
    where T: FieldElement + FiniteField<T> + AccumulatorBuilder<T>
{
    
    let trace_length = op_codes.len();

    // allocate space for the registers
    let mut registers = vec![
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
        filled_vector(trace_length, domain_size, T::ZERO),
    ];
    assert!(registers.len() == T::ACC_STATE_WIDTH, "inconsistent number of opcode accumulator registers");

    let acc = T::get_accumulator(1);
    let mut state = vec![T::ZERO; T::ACC_STATE_WIDTH];
    for i in 0..(op_codes.len() - 1) {

        // add op_code into the accumulator
        acc.apply_round(&mut state, op_codes[i], i);

        // copy updated state into registers for the next step
        for j in 0..T::ACC_STATE_WIDTH {
            registers[j][i + 1] = state[j];
        }
    }

    return registers;
}

/// Sets the op_bits registers at the specified `step` to the binary decomposition
/// of the `op_code` parameter.
fn set_op_bits<T>(op_bits: &mut Vec<Vec<T>>, op_code: u8, step: usize)
    where T: FieldElement + FiniteField<T>
{
    for i in 0..op_bits.len() {
        op_bits[i][step] = T::from((op_code >> i) & 1);
    }
}