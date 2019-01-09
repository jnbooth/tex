extern crate tex;

fn main() {
    tex::env::load();
    let result = tex::run();
    println!("{:?}", result);
}
