mod util;
fn main() {
    if util::scan_port::tcp(61000) == true {
        println!("True!");
    }
}
