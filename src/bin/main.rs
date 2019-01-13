extern crate tex;

fn main() {
    tex::env::load();
    match std::env::args().nth(1) {
        Some(ref s) if s == "--offline" || s == "-o" => println!("{:?}", tex::offline()),
        Some(s) => println!("Unknown flag {}", s),
        _ => println!("{:?}", tex::run()),
    };
}
