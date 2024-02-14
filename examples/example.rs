use lid::configs::{new_distributed, new_random};

fn main() {
    // `new_distributed` gives a low randomness LID instance with the default size of 20 bytes.
    // Don't use it if you don't want people potentially guessing next IDs.
    let mut lid = new_distributed();
    println!("{}", lid.generate());

    // `new_random` gives a high randomness LID instance.
    // Generic parameters are prefix and sequence length, combined is the total size of the id.
    let mut lid = new_random::<6, 6>();
    println!("{}", lid.generate());

    // This uses static Mutex backed instances of the generators above.
    // You must enable the `easy` feature to use these.
    println!("{}", lid::easy::generate_distributed());
    println!("{}", lid::easy::generate_random());
}
