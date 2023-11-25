use lid::LID;

fn main() {
    // This uses the global Mutex LID instance. It's slightly slower due to the Mutex.
    println!("{}", lid::generate_lid());

    let mut lid = LID::default();
    println!("{}", lid.generate());
}