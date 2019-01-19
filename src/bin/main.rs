extern crate tex;

fn main() {
    tex::load();
    match std::env::args().nth(1) {
        None => println!("{:?}", tex::run()),
        Some(s) => match s.as_str() {
            "--offline"  => println!("{:?}", tex::offline()),
            "-o"         => println!("{:?}", tex::offline()),
            _            => println!("Unknown flag {}", s)
        }
    };
}
