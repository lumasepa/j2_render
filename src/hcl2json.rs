
use molysite::hcl::parse_hcl;

use std::io::{Write, Read, stdin, stdout};

fn main() {
    let mut data = String::new();
    stdin()
        .read_to_string(&mut data)
        .expect("Error reading from stdin");
    let value = parse_hcl(&data)
        .expect("Error parsing hcl/tf/tfvars");
    let value = value
        .to_string();
    stdout()
        .write_all(value.as_ref())
        .expect("Error writing to stdout");
}
