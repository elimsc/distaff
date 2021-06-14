use distaff::{self, ProgramInputs, assembly, ProofOptions };

fn main() {
    let program = assembly::compile("
    begin
        push.3
        push.3
        xor32
    end").unwrap();
    // let's execute it
    let (outputs, proof) = distaff::execute(
        &program,
        &ProgramInputs::none(),     // we won't provide any inputs
        1,                          // we'll return one item from the stack
        &ProofOptions::default());  // we'll be using default options

    // the output should be 0
    assert_eq!(vec![0], outputs);
}