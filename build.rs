extern crate lalrpop;

// use lalrpop::Configuration;
// use std::path::Path;

fn main() {
    lalrpop::process_root().unwrap();
    //let parse_dir = Path::new("src");
    //Configuration::new().process_dir(parse_dir).unwrap()
}
