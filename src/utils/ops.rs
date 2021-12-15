// 1001 => 100 1
// n: 操作的元素是多少位
pub fn separate_last_bit(n: usize) -> String {
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

// x bit_xor 2^n-1
pub fn bit_not(n: usize) -> String {
    let mut s = String::new();
    s = format!("
    push.{}
    ", u128::pow(2, n as u32) - 1);
    s = format!("{}{}", s, bit_op(n, "xor"));
    s
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

// 1101 => 1110
pub fn rtr(n: usize, count: usize) -> String {
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

// 1101 => 1011
fn rtl(n: usize, count: usize) -> String {
    rtr(n, n - count)
}