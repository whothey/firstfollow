#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate lazy_static;

mod formalgrammar;

use std::io::{ BufRead, BufReader };
use std::fs::File;
use std::env;
use formalgrammar::Grammar;
use formalgrammar::parser::GrammarParser;

fn main() {
    let files: Vec<String> = env::args().skip(1).collect();
    let grammar: Grammar;
    let mut reader: BufReader<File>;
    let mut parser: GrammarParser = GrammarParser::new();

    env_logger::init().expect("Logger could not be initialized");

    for f in files {
        reader = BufReader::new(File::open(f).unwrap());

        for l in reader.lines() {
            let line = l.unwrap();
            debug!("Reading: {}", line);

            parser.parse_line(line);
        }

        debug!("Partial Grammar:\n{}", parser.get_grammar());
    }

    grammar = parser.finish();

    println!("{}", grammar);
    println!("FIRST: {:#?}", grammar.first_set());
    println!("FOLLOW: {:#?}", grammar.follow_set());
}
