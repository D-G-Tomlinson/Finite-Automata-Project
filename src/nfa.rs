use std::fmt;
use std::collections::HashMap;

use std::convert::From;
use std::convert::TryFrom;

use crate::dfa::DFA;
use crate::dfa::DFAState;

use crate::regex::Regex;
#[derive(Clone,Debug)]
pub struct NFAState {
    pub transitions:Vec<Vec<usize>>,
    pub accepting:bool,
}

impl NFAState {
    pub fn new(transitions:Vec<Vec<usize>>,accepting:bool) -> NFAState {
		NFAState{transitions,accepting}
    }

	fn get_transition(transition:&&str,alphabet:&HashMap<char,usize>,num_states:usize) -> Result<(usize,usize),String> {
		let parts:Vec<&str> = transition.split(":").collect();
		match parts.len() {
			0 | 1 => return Err(format!("Each transition must have a ':'")),
			2 => (),
			_ => return Err(format!("Each transition must have only one ':'")),
		};
		
		// get letter
		let chars:Vec<char> = parts[0].chars().collect();
		let transition_num:usize = match chars.len() {
			0 => 0,
			1 => {
				if alphabet.contains_key(&chars[0]) {
					alphabet[&chars[0]] + 1 as usize//must be plus one because zero represents jump
				} else {
					return Err(format!("The transition letter must be in the alphabet"));
				}
			},
			_ => return Err(format!("Each transition must have one or no letters, not multiple"))
		};
		
			// get next state
		let value:usize = match parts[1].parse::<usize>() {
			Ok(n) => n,
			Err(_) => return Err(format!("Next state must be a number"))
		};
		if value>num_states {
			return Err(format!("The next state value is too big"));
		} else if value == 0 {
			return Err(format!("The next state cannot be zero, states are indexed from 1"))
		}
		let value = value - 1;
		return Ok((transition_num,value));
	}
	
	pub fn from_line(line:&String,alphabet:&HashMap<char,usize>,num_states:usize) -> Result<NFAState,String> {
		let comma_split:Vec<&str> = line.split(",").collect();
		let num_parts = comma_split.len();
		let mut transitions:Vec<Vec<usize>> = vec![Vec::new();alphabet.len()+1]; // +1 because of the empty letter, the jump, is not in the alphabet but does have transitions

		for transition in (&comma_split[0..num_parts-1]).into_iter() {
			let transition_num:usize;
			let value:usize;

			match NFAState::get_transition(transition,alphabet,num_states) {
				Ok(result) =>	(transition_num,value) = result,
				Err(e) => return Err(e)
			}
			
			if !transitions[transition_num].contains(&value) {
				transitions[transition_num].push(value);
			}
		}
		let accepting:bool = match comma_split[num_parts -1].parse() {
			Ok(a) => a,
			Err(_)  => return Err(format!("Poorly formatted accepting value. Line is {}, value is{}",line,comma_split[num_parts -1]))
		};
		for i in 0..transitions.len() {
			transitions[i].sort();
		}
		return Ok(NFAState::new(transitions,accepting));
	}
	fn to_string(&self, alphabet:&str) -> String {
		let mut output:String = String::new();
		for i in 0..alphabet.len() {
			let letter = alphabet.chars().nth(i).unwrap();
			for goal in &self.transitions[i+1] {
				output.push(letter);
				output.push(':');
				output.push_str(&(goal+1).to_string());
				output.push(',');
			}			
		}
		for goal in &self.transitions[0] {
			output.push(':');
			output.push_str(&(goal+1).to_string());
			output.push(',');
		}		
		output.push_str(&self.accepting.to_string());
		return output;
	}
}

impl From<&DFAState> for NFAState {
	fn from(dfastate:&DFAState) -> Self {
		let mut transitions:Vec<Vec<usize>> = Vec::new();
		transitions.push(Vec::new());
		for next in &dfastate.transitions {
			transitions.push(vec![*next]);
		}
		return NFAState::new(transitions,dfastate.accepting);

	}
}

#[derive(Clone,Debug)]
pub struct NFA {
	pub states:Vec<NFAState>,
	pub starting:usize,
	pub alphabet:String
}

impl NFA {
	pub fn new(states:Vec<NFAState>,starting:usize,alphabet:String) -> NFA {
		NFA{states, starting, alphabet}
	}
	pub fn get_never_accept(alphabet:String) -> NFA {
		let states = vec![NFAState::new(vec![],false)];
		let starting = 0;
		return NFA::new(states,starting,alphabet);
	}
	
	pub fn get_accept_empty(alphabet:String) -> Result<NFA,String> {
		let state = NFAState::new(vec![Vec::<usize>::new();alphabet.len()+1],true);
		return Ok(NFA::new(vec![state],0,alphabet));
	}
	pub fn make_kstar(&mut self) {
		let len = self.states.len();
		for s in &mut self.states {
			if s.accepting {
				s.transitions[0].push(len);
				s.accepting = false
			}
		}
		let mut new_transitions:Vec<Vec<usize>> = vec![Vec::new();self.alphabet.len()+1];
		new_transitions[0].push(self.starting);
		self.states.push(NFAState::new(new_transitions,true));
		self.starting = len;
	}
	pub fn get_accept_single(i:usize,alphabet:String) -> Result<NFA,String> {
		let mut transitions = vec![Vec::<usize>::new();alphabet.len()+1];
		transitions[i+1] = vec![1];
		let start = NFAState::new(transitions,false);
		let end = NFAState::new(vec![Vec::<usize>::new();alphabet.len()+1],true);
		return Ok(NFA::new(vec![start,end],0,alphabet));
	}

	pub fn concat(r1:&mut NFA, r2:&mut NFA) -> Result<NFA, String> {
		if r1.alphabet != r2.alphabet {
			return Err(format!("Alphabet {:#?} does not match alphabet {:#?}",r1.alphabet,r2.alphabet));
		}
		let num_states = r1.states.len();
		let second_start = r2.starting + num_states;
		
		for state in &mut r1.states {
			if state.accepting {
				state.transitions[0].push(second_start);
				state.accepting = false;
			}
		}
		
		for state in &mut r2.states {
			for t in &mut state.transitions {
				for i in t {
					*i = *i + num_states;
				}
			}
		}
		r1.states.append(&mut r2.states);
		return Ok((*r1).clone());		
	}

	pub fn or(r1:&mut NFA, r2:&mut NFA) -> Result<NFA, String> {
		let num_states = r1.states.len();
		let second_start = r2.starting + num_states;
		
		for state in &mut r2.states {
			for t in &mut state.transitions {
				for i in t {
					*i = *i + num_states;
				}
			}
		}
		r1.states.append(&mut r2.states);
		let mut new_transitions = vec![Vec::<usize>::new();r1.alphabet.len()+1];
		new_transitions[0].push(r1.starting);
		new_transitions[0].push(second_start);
		
		r1.starting = r1.states.len();
		r1.states.push(NFAState::new(new_transitions,false));
		return Ok((*r1).clone());
	}
	pub fn to_regex(&self) -> Regex {
		return crate::int_nfa_reg::nfa_to_regex(&self);
	}
}

impl From<&DFA> for NFA {
	fn from(dfa:&DFA) -> Self {
		let states:Vec<NFAState> = (&dfa.states).into_iter().map(|s| NFAState::from(s)).collect();
		return NFA::new(states,dfa.starting,dfa.alphabet.clone());
	}
}

impl TryFrom<Vec<String>> for NFA {
	type Error=String;
	fn try_from(lines:Vec<String>) -> Result<Self,Self::Error> {
		let num_lines = lines.len();
		if num_lines < 3 {
			return Err(format!("Input file is too short"))
		}
		let alphabet:String = match crate::get_alphabet(&lines[0]){
			Err(e) => return Err(e),
			Ok(am) => am
		};
		let alphabet_hm = crate::get_alphabet_hm(&lines[0]);
		let starting = match lines[1].parse::<usize>() {
			Ok(n) => {
				if n <= num_lines - 2 && n>0 {
					n-1
				} else {
					return Err(format!("Starting number is not suitable"))
				}
			},
			Err(_) => return Err(format!("Starting is not a number"))
		};
		let state_lines = &lines[2..];
		let mut states:Vec<NFAState> = Vec::new();
//		states.push(NFAState::new(Vec::new(),false,Vec::new()));//dummy state to represent the bin
		for line in state_lines{
			states.push(match NFAState::from_line(line,&alphabet_hm,num_lines-2) {
				Ok(nfastate) => nfastate,
				Err(e) => return Err(e)
			});
		}
		return Ok(NFA::new(states,starting,alphabet));
	}

}

impl fmt::Display for NFA {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut output: String = String::new();
		output.push_str(&self.alphabet);
		output.push('\n');
		output.push_str(&(self.starting+1).to_string());
		for state in &self.states {
			output.push('\n');
			output.push_str(&state.to_string(&self.alphabet));
		}
		write!(f,"{}",output)
	}
}
