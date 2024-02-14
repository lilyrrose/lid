use lid::LID;

fn main() {
    // Custom ID size.
    let mut lid = LID::<12, 8>::new();
    println!("{:?}", lid.generate());
}
