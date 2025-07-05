use std::collections::HashMap;
use crate::Rslt;
use crate::dfa::DFA;
use crate::dfa::DFAState;

#[derive(Clone)]
pub struct NFAState {
    transitions:Vec<Vec<usize>>,
    accepting:bool
}

impl NFAState {
    pub fn new(transitions:Vec<Vec<usize>>,accepting:bool) -> NFAState {
		return NFAState{transitions,accepting};
    }
}

pub struct NFA {
	states:Vec<NFAState>,
	starting:usize,
	alphabet:String
}

impl NFA {
	pub fn new(states:Vec<NFAState>,starting:usize,alphabet:String) -> NFA {
		return NFA{states, starting, alphabet};
	}
	pub fn from_lines(lines:Vec<String>) -> Result<NFA,String> {
		return Err(format!("not finished yet"));
	}

	pub fn run(&self,input_word:Option<&str>, output_dfa:Option<&str>) -> Rslt {
		return Rslt::Err(format!("not finished yet"));
	}
}

pub fn nfa_option(lines:Vec<String>, input_word:Option<&str>, output_dfa:Option<&str>) -> Rslt {
    let nfa: NFA;
    match NFA::from_lines(lines) {
		Ok(n) => nfa = n,
		Err(e) => return Rslt::Err(e)
    }
    return nfa.run(input_word, output_dfa);
}

