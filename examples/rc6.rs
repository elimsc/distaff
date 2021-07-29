use distaff::{self, ProgramInputs, assembly, ProofOptions };
use std::time::SystemTime;

fn main() {
    let _commented_programe = "
    begin
        // B = B + s[0]
        read.ab
        add
        // D = D + s[1]
        read.ab
        add
        // D B

        // t = (B * (2 * B + 1)) <<< 5
        pick.1
        dup.1 // B B D B
        push.2
        mul
        push.1
        add
        mul
        push.5
        rotateleft32
        // t D B

        // u = (D * (2 * D + 1)) <<< 5
        pick.1
        dup.1 // D D t D B
        push.2
        mul
        push.1
        add
        mul
        push.5
        rotateleft32
        // u t D B

        read.ab // A s[2] u t D B
        pick.3
        xor32
        pick.2
        rotateleft32
        add
        // A u t D B

        roll.4
        swap.2
        // u t D A B

        read.ab // C s[3] u t D A B
        pick.2
        xor32
        pick.3
        rotateleft32
        add
        // C u t D A B

        roll.4
        swap.2
        // u t D C A B

        drop.2
        // D C A B, C B D A

        roll.4 // A C B D
        swap.2 // B D A C

        // start loop, i = 2 -> r
        dup.2 // B B B D A C
        push.2
        mul
        push.1
        add
        mul
        push.5
        rotateleft32 // t B D A C

        pick.2
        dup.1 // D D t B D A C
        push.2
        mul
        push.1
        add
        mul
        push.5
        rotateleft32 // u t B D A C

        pad.2 // 0 0 u t B D A C
        swap.2 // u t 0 0 B D A C
        roll.8
        roll.8 // A C u t 0 0 B D
        pick.3 // t A C u t 0 0 B D
        xor32 // t^A C u t 0 0 B D
        pick.2 // u t^A C u t 0 0 B D
        rotateleft32
        read
        add // A C u t 0 0 B D

        swap.1 // C A u t ...
        pick.2 // u C A u t ...
        xor32 // u^C A u t ...
        pick.3
        rotateleft32
        read
        add // C A u t 0 0 B D
        swap.2
        drop.2 // C A 0 0 B D
        swap.2
        drop.2 // C A B D, B D A C
        // end loop

        swap.2 // A C B D
        read
        add // A C B D
        swap.1
        read
        add // C A B D

        roll.4 // D C A B
        swap.1 // C D A B
        swap.2 // A B C D


    end
    ";

    let r = 20;
    let program = assembly::compile(&format!("
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
    end", r-1)
    ).unwrap();

    // let program = assembly::compile("
    // begin
    //     read.ab
    //     add
    //     read.ab
    //     add
    //     push.32
    //     truncate
    //
    //
    // end").unwrap();

    // secret_b: B D A C

    let inputs = ProgramInputs::new(&[],
                                    &[696990083,3196702396,910905877,3209921037,3106564365,2328944015,2734557017,58907362,3850524052,92613717,3608639738,669101535,3968880884,1720448742,3034689109,1612621259,3763384689,508198172,2967861735,2428379322,4153311602,3828858958,2859885952,1957143118,456665587,2408810598,3024553074,3945429419,3671610182,3972435254,3941627220,4091825005,4003320023,1121966248,825074983,1439490815,1675080381,2016917173,3638930516,3894899950,851050727,3795796176,2517894528,946057899],
                                    &[2003195204,4293844428,857870592,3148519816]);
    // let's execute it
    let mut start = SystemTime::now();
    let (outputs, proof) = distaff::execute(
        &program,
        &inputs,     // we won't provide any inputs
        4,                          // we'll return one item from the stack
        &ProofOptions::default());  // we'll be using default options
    println!("execution time: {:?}", SystemTime::now().duration_since(start));


    // assert_eq!(vec![1130480137,572160948,3634114028,1944429752], outputs);
    // assert_eq!(vec![2044764450, 42172886, 2223517021, 3216635275], outputs);
    assert_eq!(vec![10634699042, 4337140182, 6518484317, 3216635275], outputs);

    start = SystemTime::now();
    match distaff::verify(program.hash(), &[], &[10634699042, 4337140182, 6518484317, 3216635275], &proof) {
        Ok(_) => println!("Execution verified!"),
        Err(msg) => println!("Execution verification failed: {}", msg)
    }
    println!("verify time: {:?}", SystemTime::now().duration_since(start));
}

// execution time: Ok(1.325191s)
// Execution verified!
// verify time: Ok(3.815ms)
