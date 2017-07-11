use std::collections::{ HashMap, HashSet };
use std::fmt;
use std::fmt::Write;
use std::convert::From;

mod symbol;
pub mod parser;

use self::symbol::Symbol;

pub type Rule = Vec<Symbol>;
pub type State = String;
pub type FirstSet  = HashSet<char>;
pub type FirstMap  = HashMap<State, FirstSet>;
pub type FollowSet = HashSet<char>;
pub type FollowMap = HashMap<State, FollowSet>;

const EPSILON_CHAR: char = '§';

lazy_static! {
    static ref FIRST_EPSILON: Symbol = Symbol::Terminal(EPSILON_CHAR.to_string());
}

pub struct Grammar {
    states: HashMap<State, bool>,
    map: HashMap<State, HashSet<Rule>>,
    initial: State
}

impl Grammar {
    pub fn new() -> Self {
        let mut states_map  = HashMap::new();
        let mut grammar_map = HashMap::new();

        states_map.insert("S".to_string(), false);
        grammar_map.insert("S".to_string(), HashSet::new());

        Grammar {
            states: states_map,
            map: grammar_map,
            initial: "S".to_string()
        }
    }

    pub fn create_state(&mut self, n: String) {
        let name = n.trim().to_string();

        self.states.insert(name.clone(), false);
        self.map.insert(name, HashSet::new());
    }

    pub fn add_rule_to(&mut self, state: &String, rule: Rule) -> Result<(), String> {
        if let Some(rules) = self.map.get_mut(state) {
            rules.insert(rule);
            Ok(())
        } else {
            Err("No state named ".to_string() + state)
        }
    }

    pub fn first_set(&self) -> HashMap<State, HashSet<char>> {
        let mut first: FirstMap = HashMap::new();
        let mut first_updated = true;

        // Loops while there is updates in any FirstSets of FirstMap
        while first_updated == true {
            first_updated = false;

            for (key, rules) in &self.map {
                first.entry(key.to_owned()).or_insert(HashSet::new());

                for rule in rules {
                    // Parse possible first symbols
                    let mut symbols: Vec<&Symbol> = Vec::new();
                    let mut depth: usize = 0;

                    if self.states[key] {
                        symbols.push(&FIRST_EPSILON);
                    }

                    for symbol in rule {
                        // Always add the reference to symbol
                        symbols.push(symbol);

                        match symbol {
                            &Symbol::NonTerminal(ref name) => {
                                // If state accept, then let the iteration continue, else, stops the
                                // iteration
                                if self.states[name] {
                                    if depth == rule.len() - 1 {
                                        symbols.push(&FIRST_EPSILON);
                                        break;
                                    } else {
                                        depth += 1;
                                    }
                                } else { break }
                            },
                            // Also, if symbol is a terminal, it is already on Symbols vec and so will
                            // be parsed after
                            _ => break
                        }
                    }

                    // With all the proper symbols parsed, just iterate and solve it to FirstMap
                    for symbol in symbols {
                        match symbol {
                            &Symbol::Terminal(ref content) => {
                                let mut set = first.get_mut(key).unwrap(); 
                                let     old = &set.clone();

                                set.insert(content.chars().nth(0).unwrap());

                                if set != old { first_updated = true }
                            },
                            &Symbol::NonTerminal(ref name) => {
                                if first.contains_key(name) {
                                    // Current set
                                    let     set    = first[key].clone();
                                    // Associated State FirstSet
                                    let mut target = first[name].clone();

                                    // Do not get Epsilon from target FirstSet
                                    target.remove(&EPSILON_CHAR);

                                    let union = set.union(&target)
                                        .cloned()
                                        .collect::<HashSet<_>>();

                                    first.insert(key.to_owned(), union);

                                    if set != first[key] { first_updated = true }
                                }
                            }
                        }
                    }
                }
            }
        }

        first
    }

    pub fn follow_set(&self) -> FollowMap {
        unimplemented!()
    }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf: String = String::new();
        let mut ordered_states: Vec<_> = self.states.keys().collect();

        ordered_states.sort();

        // Print initial state
        {
            write!(buf, "<{}> ::= ", self.initial).unwrap();

            for rule in &self.map[&self.initial] {
                for sym in rule {
                    write!(buf, "{}", sym).unwrap();
                }

                write!(buf, " | ").unwrap();
            }

            // Accept?
            if self.states[&self.initial] {
                write!(buf, "<>\n").unwrap();
            } else {
                write!(buf, "\n").unwrap();
            }
        }

        for s in ordered_states {
            if s == &self.initial { continue }

            write!(buf, "<{}> ::= ", s).unwrap();

            for rule in &self.map[s] {
                for sym in rule {
                    write!(buf, "{}", sym).unwrap();
                }

                write!(buf, " | ").unwrap();
            }

            // Accept?
            if self.states[s] {
                write!(buf, " <>\n").unwrap();
            } else {
                write!(buf, "\n").unwrap();
            }
        }

        write!(f, "{}", buf)
    }
}

impl From<parser::GrammarParser> for Grammar {
    fn from(parser: parser::GrammarParser) -> Self {
        parser.finish()
    }
}

impl From<String> for Grammar {
    fn from(source: String) -> Self {
        let mut parser = parser::GrammarParser::new();

        for line in source.lines() {
            parser.parse_line(line.to_string());
        }

        Self::from(parser)
    }
}
