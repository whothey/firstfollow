use super::{ Grammar, Rule, Symbol };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserState {
    // Left part of <AS> ::= ...
    // Until read '<'
    State,
    // Reading the contents of <>, hold the old state
    // Until read '>'
    StateName,
    // Store the name read in <>, which is in the buffer
    StoreState,
    Defs, // Right part of <AS> ::= ab<A> | b<B>
    DefsNonTerminal // <A>, <B>
}

pub struct GrammarParser {
    buffer: String,
    current_reading: String,
    temp_rule: Rule,
    read_state: ParserState,
    grammar: Grammar
}

impl GrammarParser {
    pub fn new() -> Self {
        GrammarParser {
            buffer: String::new(),
            current_reading: String::new(),
            temp_rule: Vec::new(),
            read_state: ParserState::State,
            grammar: Grammar::new()
        }
    }

    pub fn parse_line(&mut self, line: String) {
        for ch in line.chars() {
            match self.read_state {
                ParserState::State => {
                    match ch {
                        '<' => self.read_state = ParserState::StateName,
                        _  => ()
                    }
                },
                ParserState::StateName => {
                    if ch != '>' && ch != ':' && ch != '=' {
                        self.buffer.push(ch);
                    } else {
                        self.read_state = ParserState::StoreState;
                    }
                },
                ParserState::StoreState => {
                    let state_name = self.buffer.drain(..)
                        .collect::<String>()
                        .trim()
                        .to_string();

                    self.current_reading = state_name.clone();
                    debug!("CREATE STATE: {}", state_name);
                    self.grammar.create_state(state_name);
                    self.read_state = ParserState::Defs;
                },
                ParserState::Defs if ch != ':' && ch != '=' => {
                    if ch == '|' || ch == '\n' {
                        if self.buffer.len() > 0 {
                            self.store_buffer_as_terminal();
                        }
                        debug!("PUSH: {:?}", self.temp_rule);
                        self.store_rule();
                    } else if ch == '<' {
                        if self.buffer.len() > 0 {
                            self.store_buffer_as_terminal();
                        }

                        self.read_state = ParserState::DefsNonTerminal;
                    } else {
                        self.buffer.push(ch)
                    }
                },
                ParserState::DefsNonTerminal => {
                    if ch != '>' {
                        self.buffer.push(ch);
                    } else {
                        // Finish reading a nonterminal
                        self.store_buffer_as_nonterminal();
                        self.read_state = ParserState::Defs;
                    }
                },
                _ => ()
            }
        }

        // Before \n
        if self.buffer.len() > 0 {
            self.store_buffer_as_terminal();
        }
        debug!("PUSH: {:?}", self.temp_rule);
        self.store_rule();

        self.read_state = ParserState::State;
    }

    fn store_buffer_as_terminal(&mut self) {
        let tname = self.buffer.drain(..)
            .collect::<String>()
            .trim()
            .to_string();

        if tname.len() == 0 { return; }

        let sym = Symbol::Terminal(tname);

        debug!("Storing Terminal: {}", sym);

        self.temp_rule.push(sym);
    }

    fn store_buffer_as_nonterminal(&mut self) {
        let ntname = self.buffer.drain(..)
            .collect::<String>()
            .trim()
            .to_string();

        let sym = if ntname.len() == 0 {
            self.grammar.states.insert(self.current_reading.clone(), true);
            return
        } else {
            Symbol::NonTerminal(ntname)
        };

        debug!("Storing NonTerminal: {}", sym);

        self.temp_rule.push(sym);
    }

    fn store_rule(&mut self) {
        let rule: Vec<_> = self.temp_rule.drain(..).collect();

        if rule.len() > 0 {
            self.grammar.add_rule_to(&self.current_reading, rule)
                .expect("WTF");
        }
    }

    pub fn get_grammar(&self) -> &Grammar {
        &self.grammar
    }

    pub fn finish(self) -> Grammar {
        self.grammar
    }
}
