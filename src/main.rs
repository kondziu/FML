#[macro_use] extern crate lalrpop_util;

lalrpop_mod!(pub fml); // load module synthesized by LALRPOP

fn main() {
    println!("hi")
}
