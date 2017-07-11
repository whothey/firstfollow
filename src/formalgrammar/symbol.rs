use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    Terminal(String),
    NonTerminal(String)
}

impl Symbol {
    pub fn is_terminal(&self) -> bool {
        match *self {
            Symbol::Terminal(_) => true,
            _ => false
        }
    }

    pub fn is_nonterminal(&self) -> bool {
        match *self {
            Symbol::NonTerminal(_) => true,
            _ => false
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Symbol::NonTerminal(ref s) => write!(f, "<{}>", s),
            &Symbol::Terminal(ref s) => write!(f, "{}", s)
        }
    }
}
