use std::mem;

#[repr(C)]
enum Mutation {
    RemoveStep(usize, usize, usize),
    AddStep(usize, usize, usize),
    ModifyParam(usize, usize, usize, u8),
}

const MUTATION_SIZE: usize = mem::size_of::<Mutation>();
type MutationBuffer = [u8; MUTATION_SIZE];

fn main() {
    // mem::transmute(e)
    // let buffer: MutationBuffer = [0; MUTATION_SIZE];

    let mutation = Mutation::RemoveStep(1, 3, 2);

    let num = unsafe { std::mem::transmute::<Mutation, MutationBuffer>(mutation) };

    println!("{:?}", num);
}
