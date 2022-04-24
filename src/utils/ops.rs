
// 1001 => 100 1
// n: 操作的元素是多少位
fn separate_last_bit(n: usize) -> String {
    let mut size = n;
    if n < 4 {
        size = 4;
    }
    format!("
    dup.1
    isodd.{}
    swap.1
    pick.1
    sub
    push.2
    div
    ", size)
}

// truncate(2): 1011 => 11
pub fn truncate(n: usize, count: usize) -> String {
    // stack: 1011
    // push.0: 0 1011

    // swap.1: 1011 x
    // sep: 101 1 x
    // push.0: 0 101 1 x
    // swap.2: 1 x 0 101
    // push.2**0: 1 1 x 0 101
    // mul: 1 x 0 101
    // add: 1+x 0 101
    // swap.1: 0 1+x 101
    // drop.1: 1+x 101

    // swap.1: 101 1+x
    // drop.1: 1+x
    let mut s = String::new();

    s += "
    push.0
    ";

    for i in 0..count {
        let substr = format!("
    swap.1
    {}
    push.0
    swap.2
    push.{}
    mul
    add
    swap.1
    drop.1
    ", separate_last_bit(n-i), u128::pow(2, i as u32));
    // TODO: separate_last_bit的参数应该如何设置

        s = format!("{}{}", s, substr)
    }

    s += "
    swap.1
    drop.1
    ";

    s
}

pub fn bit_not(n: usize) -> String {
    // stack: 1011
    // truncate(3): 011
    // push.2**3-1: 7 011
    // swap.1: 011 111
    // sub: 100
    format!("
    {}
    push.{}
    swap.1
    sub
    ", truncate(n, n), u128::pow(2, n as u32)-1)
}

// 1010 1101 => 0111
// 1010 1101
// push.0: x 1010 1101
// swap.1: 1010 x 1101
// sep: 101 0 x 1101
// roll.4: 1101 101 0 x
// sep: 110 1 101 0 x
// swap.1: 1 110 101 0 x
// roll.4: 0 1 110 101 x
// ne: 1 110 101 x  (这里根据xor, and, or更改)
// push.2**0: 1 1 110 101 x
// mul: 1 110 101 x
// roll.4: x 1 110 101
// add: x+1 110 101
fn bit_op(n: usize, op: &str) -> String {
    // op: and, or, xor
    let mut s = String::new();

    let mut act = op;
    if op == "xor" {
        act = "ne"
    }

    s += "
    push.0
    ";

    for i in 0..n {
        let substr = format!("
    swap.1
    {}
    roll.4
    {}
    swap.1
    roll.4
    {}
    push.{}
    mul
    roll.4
    add
    ", separate_last_bit(n-i), separate_last_bit(n-i), act, u128::pow(2, i as u32));

        s = format!("{}{}", s, substr)
    }

    s += "
    swap.1
    drop.1
    swap.1
    drop.1
    ";

    s
}

pub fn bit_xor(n: usize) -> String {
    return bit_op(n, "xor");
}

pub fn bit_and(n: usize) -> String {
    return bit_op(n, "and");
}

pub fn bit_or(n: usize) -> String {
    return bit_op(n, "or");
}



// 1101 => 11010
pub fn shl(n: usize, count: usize) -> String {
    if count == 1 {
        return format!("
    push.2
    mul
    ");
    }
    format!("
    repeat.{}
    push.2
    mul
    end
    ", count)
}

// 1101 => 110
pub fn shr(n: usize, count: usize) -> String {
    if count == 1 {
        return format!("
    {}
    swap.1
    drop.1
    ", separate_last_bit(n));
    }
    format!("
    repeat.{}
        {}
        swap.1
        drop.1
    end
    ", count, separate_last_bit(n))
}

// stack: y x
// stack: x % y
fn mod1(n: usize) -> String {
    // stack: y x
    // dup.2: y x y x
    // lt.n: 1 y x
    // while.true: y x
    // swap.1: x y
    // dup.2: x y x y
    // swap.1: y x x y
    // sub: x-y x y
    // swap.1: x x-y y
    // drop.1: x-y y
    // swap.1 y x-y
    // dup.2: y x-y y x-y
    // lt.n
    // end: y x-ty
    // pick.1: x-ty y x-ty
    // ne: 0/1 x-ty
    // mut: 0 or x-ty
    format!("
    dup.2
    lt.{}
    while.true
        swap.1
        dup.2
        swap.1
        sub
        swap.1
        drop.1
        swap.1
        dup.2
        lt.{}
    end
    pick.1
    ne
    mul
    ", 128, 128)
    //TODO
}

// x(32bit) <<< count
// n: bits of count
pub fn rtl32_var(n: usize) -> String {
    // stack: count x
    // truncate(5): count1(<32) x
    // push.32
    // swap.1
    // sub
    // rtr_helper
    format!("
    {}
    push.32
    swap.1
    sub
    {}
    ", truncate(n, 5), rtr_var_helper(32))
}

// x(32bit) >>> count
pub fn rtr32_var(n: usize) -> String {
    // stack: count x
    // truncate(5): count1(<32) x
    // rtr_helper
    format!("
    {}
    {}
    ", truncate(n, 5), rtr_var_helper(32))
}

// x(64bit) <<< count
pub fn rtl64_var(n: usize) -> String {
    // stack: count x
    // truncate(5): count1(<32) x
    // push.32
    // swap.1
    // sub
    // rtr_helper
    format!("
    {}
    push.64
    swap.1
    sub
    {}
    ", truncate(n, 6), rtr_var_helper(32))
}

// x(64bit) >>> count
pub fn rtr64_var(n: usize) -> String {
    // stack: count x
    // truncate(6): count1(<64) x
    // rtr_helper
    format!("
    {}
    {}
    ", truncate(n, 6), rtr_var_helper(32))
}

// 循环右移，不考虑移动的次数是否大于n
fn rtr_var_helper(n: usize) -> String {
    // stack: 5 x (用户需要保证 x < x**n)
    // dup.1: 5 5 x
    // push.0: 0 5 5 x
    // lt.7: 1 5 x
    // while.true: 5 x
    // swap.1: x 5
    // rtr(n,1): x1 5
    // swap.1: 5 x1
    // push.1: 1 5 x1
    // sub: 4 x1
    // dup.1: 4 4 x1
    // push.0: 0 4 4 x1
    // lt.7: 1 4 x1
    // end: 0 xt
    // drop.1: xt
    // TODO: 128?
    format!("
    dup.1
    push.0
    lt.{}
    while.true
        swap.1
        {}
        swap.1
        push.1
        sub
        dup.1
        push.0
        lt.{}
    end
    drop.1
    ",  n, rtr_const(n, 1), n)
}

// 1101 => 1110
pub fn rtr_const(n: usize, count: usize) -> String {
    if count == 1 {
        return format!("
    {}
    swap.1
    push.{}
    mul
    add
    ", separate_last_bit(n), u128::pow(2, (n-1) as u32));
    }
    format!("
    repeat.{}
        {}
        swap.1
        push.{}
        mul
        add
    end
    ", count, separate_last_bit(n), u128::pow(2, (n-1) as u32))
}

// rtl(4, 1): 1101 => 1011
pub fn rtl_const(n: usize, count: usize) -> String {
    rtr_const(n, n - count)
}



#[cfg(test)]
mod tests {
    use crate::{assembly, ProgramInputs, ProofOptions, execute, verify};
    use crate::utils::ops::*;

    fn test_helper(program_str: String, expected_outputs: Vec<u128>) {
        let program = assembly::compile(&program_str).unwrap();

        let (outputs, proof) = execute(
            &program,
            &ProgramInputs::none(), // we won't provide any inputs
            expected_outputs.len(),       // we'll return one item from the stack
            &ProofOptions::default(),
        );

        assert_eq!(
            expected_outputs,
            outputs
        );

        verify(
            program.hash(),
            &[],
            &expected_outputs,
            &proof,
        ).unwrap();
    }

    #[test]
    fn test_bit_xor() {
        let program_str = format!("
    begin
        push.11
        push.200
        {}
    end
    ", bit_xor(8));

        let expected_outputs = vec![195];
        test_helper(program_str, expected_outputs);
    }

    #[test]
    fn test_mod1() {
        let program_str = format!("
    begin
        push.11
        push.10
        {}
    end
    ", mod1(32));

        let expected_outputs = vec![1];
        test_helper(program_str, expected_outputs);

        let program_str1 = format!("
    begin
        push.10
        push.10
        {}
    end
    ", mod1(32));

        let expected_outputs = vec![0];
        test_helper(program_str1, expected_outputs);
    }

    #[test]
    fn test_rtr32_var() {
        let program_str = format!("
    begin
        push.10
        push.2
        {}
    end
    ", rtr32_var(4));

        let expected_outputs = vec![2147483650];
        test_helper(program_str, expected_outputs);
    }

    #[test]
    fn test_rtl32_var() {
        let program_str = format!("
    begin
        push.2147483650
        push.2
        {}
    end
    ", rtl32_var(4));

        let expected_outputs = vec![10];
        test_helper(program_str, expected_outputs);
    }

    #[test]
    fn test_truncate() {
        let program_str = format!("
    begin
        push.11
        {}
    end
    ", truncate(3, 3));

        let expected_outputs = vec![3];
        test_helper(program_str, expected_outputs);
    }

    #[test]
    fn test_bit_not() {
        let program_str = format!("
    begin
        push.11
        {}
    end
    ", bit_not(4));

        let expected_outputs = vec![4];
        test_helper(program_str, expected_outputs);
    }

}

