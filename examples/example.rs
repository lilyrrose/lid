use lid::LID;

fn main() {
    // This uses the global Mutex LID instance. It's slightly slower due to the Mutex.
    println!("{}", lid::generate_lid());

    // Custom ID size.
    let mut lid = LID::<12, 8>::new();
    println!("{}", lid.generate());
}
