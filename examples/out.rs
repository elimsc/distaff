use std::fmt::write;
use std::fs;

use distaff::{assembly, ProgramInputs, ProofOptions, StarkProof};
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};

fn main() {
    // this is our program, we compile it from assembly code
    let r = 20;
    let program = assembly::compile(&format!(
        "
    begin
        read.ab
        add
        read.ab
        add

        pick.1
        dup.1
        push.2
        mul
        push.1
        add
        mul
        push.5
        rotateleft32

        pick.1
        dup.1
        push.2
        mul
        push.1
        add
        mul
        push.5
        rotateleft32

        read.ab
        pick.3
        xor32
        pick.2
        rotateleft32
        add

        roll.4
        swap.2

        read.ab
        pick.2
        xor32
        pick.3
        rotateleft32
        add

        roll.4
        swap.2

        drop.2

        roll.4
        swap.2

        repeat.{}
            dup.2
            push.2
            mul
            push.1
            add
            mul
            push.5
            rotateleft32

            pick.2
            dup.1
            push.2
            mul
            push.1
            add
            mul
            push.5
            rotateleft32

            pad.2
            swap.2
            roll.8
            roll.8
            pick.3
            xor32
            pick.2
            rotateleft32
            read
            add

            swap.1
            pick.2
            xor32
            pick.3
            rotateleft32
            read
            add
            swap.2
            drop.2
            swap.2
            drop.2
        end

        swap.2
        read
        add
        swap.1
        read
        add

        roll.4
        swap.1
        swap.2
    end",
        r - 1
    ))
    .unwrap();

    let inputs = ProgramInputs::new(
        &[],
        &[
            696990083, 3196702396, 910905877, 3209921037, 3106564365, 2328944015, 2734557017,
            58907362, 3850524052, 92613717, 3608639738, 669101535, 3968880884, 1720448742,
            3034689109, 1612621259, 3763384689, 508198172, 2967861735, 2428379322, 4153311602,
            3828858958, 2859885952, 1957143118, 456665587, 2408810598, 3024553074, 3945429419,
            3671610182, 3972435254, 3941627220, 4091825005, 4003320023, 1121966248, 825074983,
            1439490815, 1675080381, 2016917173, 3638930516, 3894899950, 851050727, 3795796176,
            2517894528, 946057899,
        ],
        &[2003195204, 4293844428, 857870592, 3148519816],
    );

    // let's execute it
    let (outputs, proof) = distaff::execute(
        &program,
        &inputs, // we won't provide any inputs
        4,       // we'll return one item from the stack
        &ProofOptions::default(),
    ); // we'll be using default options

    let proof_str = serde_json::to_string(&proof).unwrap();
    let mut file = File::create("out.json").unwrap();
    write!(file, "{}", proof_str).unwrap();
    println!("write ok");

    let proof_str1 = fs::read_to_string("out.json").unwrap();
    let proof1: StarkProof = serde_json::from_str(&proof_str1).unwrap();
    match distaff::verify(
        program.hash(),
        &[],
        &[10634699042, 4337140182, 6518484317, 3216635275],
        &proof1,
    ) {
        Ok(_) => println!("Execution verified!"),
        Err(msg) => println!("Execution verification failed: {}", msg),
    }
}
