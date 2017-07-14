use std::collections::{ HashMap, HashSet };
use std::fmt;
use std::fmt::Write;
use std::convert::From;

mod symbol;
pub mod parser;

use self::symbol::Symbol;

pub type Rule      = Vec<Symbol>;
pub type State     = String;
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
                            &Symbol::NonTerminal(ref name) if first.contains_key(name) => {
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
                            },
                            _ => ()
                        }
                    }
                }
            }
        }

        first
    }

    pub fn follow_set(&self) -> FollowMap {
        let first_map = self.first_set();
        let mut follow_map: FollowMap = HashMap::new();
        let mut follow_changed = true;

        {
            follow_map.entry(self.initial.clone())
                .or_insert(HashSet::new())
                .insert('$');
        }

        // First Iteration: grab data using `first_map`
        for (_, rules) in &self.map {
            for rule in rules {
                // Grab next terminal to Some(StateName)
                let mut grab_to = None;

                for symbol in rule {
                    match symbol {
                        &Symbol::NonTerminal(ref name) if grab_to.is_none() => {
                            grab_to = Some(name.clone());
                        },
                        &Symbol::NonTerminal(ref name) if grab_to.is_some() => {
                            let target = grab_to.take().unwrap();
                            let union  = {
                                let set = follow_map.entry(target.clone())
                                    .or_insert(HashSet::new());

                                let mut related = first_map[name].clone();
                                related.remove(&EPSILON_CHAR);

                                related.union(set).cloned().collect::<HashSet<_>>()
                            };

                            follow_map.insert(target.clone(), union);
                            grab_to = Some(name.clone());
                        },
                        &Symbol::Terminal(ref s) if grab_to.is_some() => {
                            let target  = grab_to.take().unwrap();
                            let mut set = follow_map.entry(target)
                                .or_insert(HashSet::new());

                            set.insert(s.chars().nth(0).unwrap().clone());
                        },
                        _ => ()
                    }
                }
            }
        }

        // Grab from Follow
        while follow_changed {
            follow_changed = false;

            for (key, rules) in &self.map {
                for rule in rules {
                    // Grab next terminal to Some(StateName)
                    let mut grab_to: Option<String> = None;

                    for symbol in rule {
                        if let &Symbol::NonTerminal(ref name) = symbol {
                            if let Some(target) = grab_to.take() {
                                let set = follow_map[&target].clone();

                                let union  = {
                                    let related = follow_map[name].clone();

                                    related.union(&set).cloned().collect::<HashSet<_>>()
                                };

                                if union != set { follow_changed = true }
                                follow_map.insert(target.clone(), union);
                            }

                            grab_to = Some(name.clone());
                        } else {
                            grab_to.take();
                        }
                    }
                    // Mapping there situations:
                    // A ::= aB => Follow(B) = Follow(B) + Follow(A)
                    // and
                    // B ::= aCD => and Epsilon in First(D), then:
                    // Follow(C) = Follow(C) + Follow(B)
                    // Parse in reverse order, mix the follows if upper conditions are fullfiled
                    // and symbols are NonTerminals. Continue only if Epsilon IN First(last_read)
                    for revs in rule.iter().rev() {
                        if let &Symbol::NonTerminal(ref name) = revs {
                            let set = follow_map[name].clone();

                            let union = {
                                let related = follow_map[key].clone();

                                related.union(&set).cloned().collect::<HashSet<_>>()
                            };

                            if union != set { follow_changed = true }
                            follow_map.insert(name.clone(), union);

                            if ! first_map[name].contains(&EPSILON_CHAR) { break; }
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        follow_map
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

            for (i, rule) in self.map[&self.initial].iter().enumerate() {
                if i > 0 { write!(buf, " | ").unwrap() }

                for sym in rule {
                    write!(buf, "{}", sym).unwrap();
                }
            }

            // Accept?
            if self.states[&self.initial] {
                write!(buf, " | <>\n").unwrap();
            } else {
                write!(buf, "\n").unwrap();
            }
        }

        for s in ordered_states {
            if s == &self.initial { continue }

            write!(buf, "<{}> ::= ", s).unwrap();

            for (i, rule) in self.map[s].iter().enumerate() {
                if i > 0 { write!(buf, " | ").unwrap() }

                for sym in rule {
                    write!(buf, "{}", sym).unwrap();
                }
            }

            // Accept?
            if self.states[s] {
                write!(buf, " | <>\n").unwrap();
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
