use std::fs;
use std::time::SystemTime;

use distaff::{assembly, ProgramInputs, ProofOptions, StarkProof};
use std::fs::File;
use std::io::Write;

fn main() {
    let pub_inputs: Vec<u128> = vec![];
    let n_blocks = 16;

    let key = vec![1u128, 2]; // key size: 256bit
    let mut data = vec![1u128, 2, 3, 4]; // 512bit per block
    for _i in 0..n_blocks - 1 {
        let mut block = vec![1u128, 2, 3, 4];
        data.append(&mut block);
    }

    let mut secret_a: Vec<u128> = vec![];

    secret_a.resize(
        key.len() + data.len() + (key.len() + data.len()) * n_blocks,
        0,
    );

    secret_a[0..key.len()].copy_from_slice(&key);
    secret_a[key.len()..key.len() + data.len()].copy_from_slice(&data);
    let start = key.len() + data.len();
    for i in 0..n_blocks {
        secret_a[start + i * 6..start + i * 6 + 4].copy_from_slice(&data[i * 4..i * 4 + 4]);
        secret_a[start + i * 6 + 4..start + i * 6 + 6].copy_from_slice(&key);
    }

    let secret_b: Vec<u128> = vec![];
    // this is our program, we compile it from assembly code
    let program = assembly::compile(&format!(
        "
        begin
        read
        read
        hash

        push.0
        push.0

        repeat.{}
            read
            add
            swap.1
            read
            add
            swap.1
            hash
        end


        push.0
        push.0
        push.0
        push.0

        repeat.{}
            read
            add
            roll.4
            read
            add
            roll.4
            read
            add
            roll.4
            read
            add
            roll.4

            read
            read
            cipher
        end

    end",
        n_blocks * 2,
        n_blocks
    ))
    .unwrap();

    let inputs = ProgramInputs::new(&pub_inputs, &secret_a, &secret_b);

    let mut now = SystemTime::now();
    // let's execute it
    let (outputs, proof) = distaff::execute(
        &program,
        &inputs, // we won't provide any inputs
        8,       // we'll return one item from the stack
        &ProofOptions::default(),
    ); // we'll be using default options
    println!("proving time: {:?}", SystemTime::now().duration_since(now));

    let proof_str = serde_json::to_string(&proof).unwrap();
    let mut file = File::create("out.json").unwrap();
    write!(file, "{}", proof_str).unwrap();
    println!("write ok");

    let proof_str1 = fs::read_to_string("out.json").unwrap();
    let proof1: StarkProof = serde_json::from_str(&proof_str1).unwrap();
    now = SystemTime::now();
    match distaff::verify(program.hash(), &pub_inputs, &outputs, &proof1) {
        Ok(_) => println!("Execution verified!"),
        Err(msg) => println!("Execution verification failed: {}", msg),
    }
    println!("verify time: {:?}", SystemTime::now().duration_since(now));
}
