use std::fmt;
use std::collections::HashMap;

use crate::StateNum;
use crate::Ordered;

use std::convert::From;
use std::convert::TryFrom;

use crate::dfa::DFA;
use crate::dfa::DFAState;

use crate::regex::Regex;

use crate::Index1;
use crate::Index0;

#[derive(Clone,Debug)]
pub struct NFAState {
    pub transitions:Vec<Ordered>,
    pub accepting:bool,
}

impl NFAState {
    pub fn new(transitions:Vec<Ordered>,accepting:bool) -> NFAState {
		NFAState{transitions,accepting}
    }

	fn get_transition(transition:&&str,alphabet:&HashMap<char,Index0>,max_state:StateNum) -> Result<(Index1,StateNum),String> {
		let parts:Vec<&str> = transition.split(":").collect();
		match parts.len() {
			0 | 1 => return Err("Each transition must have a ':'".to_string()),
			2 => (),
			_ => return Err("Each transition must have only one ':'".to_string()),
		};
		
		// get letter
		let chars:Vec<char> = parts[0].chars().collect();
		let transition_num:Index1 = match chars.len() {
			0 => Index1(0),
			1 => {
				if alphabet.contains_key(&chars[0]) {
					alphabet[&chars[0]].into()//must be plus one because zero represents jump
				} else {
					return Err("The transition letter must be in the alphabet".to_string());
				}
			},
			_ => return Err("Each transition must have one or no letters, not multiple".to_string())
		};
		
			// get next state
		let value:StateNum = match parts[1].parse::<StateNum>() {
			Ok(n) => n,
			Err(_) => return Err("Next state must be a number".to_string())
		};
		if value>max_state {
			return Err("The next state value is too big".to_string());
		} else if value == 0 {
			return Err("The next state cannot be zero, states are indexed from 1".to_string())
		}
		let value = value - 1;
		return Ok((transition_num,value));
	}
	
	pub fn from_line(line:&String,alphabet:&HashMap<char,Index0>,max_state:StateNum) -> Result<NFAState,String> {
		let comma_split:Vec<&str> = line.split(",").collect();
		let num_parts = comma_split.len();
		let mut transitions:Vec<Vec<StateNum>> = vec![Vec::new();alphabet.len()+1]; // +1 because of the empty letter, the jump, is not in the alphabet but does have transitions

		for transition in (&comma_split[0..num_parts-1]).into_iter() {
			let (transition_num,value) = match NFAState::get_transition(transition,alphabet,max_state) {
				Ok(result) => result,
				Err(e) => return Err(e)
			};
			
			if !transitions[transition_num.0].contains(&value) {
				transitions[transition_num.0].push(value);
			}
		}
		let accepting:bool = match comma_split[num_parts -1].parse() {
			Ok(a) => a,
			Err(_)  => return Err(format!("Poorly formatted accepting value. Line is {}, value is{}",line,comma_split[num_parts -1]))
		};

		let mut new_transitions:Vec<Ordered> = Vec::new();
		for i in 0..transitions.len() {
			transitions[i].sort();
			new_transitions.push(Ordered(transitions[i].clone()));
		}
		let transitions = new_transitions;
		return Ok(NFAState::new(transitions,accepting));
	}
	fn to_string(&self, alphabet:&str) -> String {
		let mut output:String = String::new();
		for i in 0..alphabet.len() {
			let index:Index0 = Index0(i);//for alphabet
			let trans_num:Index1 = index.into();//for transitions list
			let letter = alphabet.chars().nth(index.0).unwrap();
			for goal in &self.transitions[trans_num.0].0 {
				output.push(letter);
				output.push(':');
				output.push_str(&(goal+1).to_string());
				output.push(',');
			}			
		}
		for goal in &self.transitions[0].0 {
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
		let mut transitions:Vec<Ordered> = Vec::new();
		transitions.push(Ordered(Vec::new()));
		for next in &dfastate.transitions {
			transitions.push(Ordered(vec![*next]));
		}
		return NFAState::new(transitions,dfastate.accepting);

	}
}

#[derive(Clone,Debug)]
pub struct NFA {
	pub states:Vec<NFAState>,
	pub starting:StateNum,
	pub alphabet:String
}

impl NFA {
	pub fn new(states:Vec<NFAState>,starting:StateNum,alphabet:String) -> NFA {
		NFA{states, starting, alphabet}
	}
	pub fn get_never_accept(alphabet:String) -> NFA {
		let states = vec![NFAState::new(vec![],false)];
		let starting = 0;
		return NFA::new(states,starting,alphabet);
	}
	
	pub fn get_accept_empty(alphabet:String) -> Result<NFA,String> {
		let state = NFAState::new(vec![Ordered(Vec::new());alphabet.len()+1],true);
		return Ok(NFA::new(vec![state],0,alphabet));
	}
	pub fn make_kstar(&mut self) {
		let len = self.states.len();
		for s in &mut self.states {
			if s.accepting {
				s.transitions[0] = s.transitions[0].join(&Ordered(vec![len]));
				s.accepting = false
			}
		}
		let mut new_transitions:Vec<Ordered> = vec![Ordered(Vec::new());self.alphabet.len()+1];
		new_transitions[0]= Ordered(vec![self.starting]);
		self.states.push(NFAState::new(new_transitions,true));
		self.starting = len;
	}
	pub fn get_accept_single(i:Index1,alphabet:String) -> Result<NFA,String> {
		let mut transitions:Vec<Ordered> = vec![Ordered(Vec::new());alphabet.len()+1];
		transitions[i.0] = Ordered(vec![1]);
		let start = NFAState::new(transitions,false);
		let end = NFAState::new(vec![Ordered(Vec::new());alphabet.len()+1],true);
		return Ok(NFA::new(vec![start,end],0,alphabet));
	}

	pub fn bump_states_append(r1:&mut NFA,r2:&mut NFA, by:StateNum) {
		for state in &mut r2.states {
			for t in &mut state.transitions {
				for i in &mut t.0 {
					*i = *i + by;
				}
			}
		}
		r1.states.append(&mut r2.states);
	}

	pub fn concat(r1:&mut NFA, r2:&mut NFA) -> Result<NFA, String> {
		if r1.alphabet != r2.alphabet {
			return Err(format!("Alphabet {:#?} does not match alphabet {:#?}",r1.alphabet,r2.alphabet));
		}
		let num_states = r1.states.len();
		let second_start = r2.starting + num_states;
		
		for state in &mut r1.states {
			if state.accepting {
				state.transitions[0] = state.transitions[0].join(&Ordered(vec![second_start]));
				state.accepting = false;
			}
		}
		Self::bump_states_append(r1,r2,num_states);
		return Ok((*r1).clone());		
	}

	pub fn or(r1:&mut NFA, r2:&mut NFA) -> Result<NFA, String> {
		let num_states = r1.states.len();
		let second_start = r2.starting + num_states;
		Self::bump_states_append(r1,r2,num_states);
		let mut new_transitions = vec![Ordered(Vec::<StateNum>::new());r1.alphabet.len()+1];

		let mut jumps = vec![r1.starting,second_start];
		jumps.sort();

		new_transitions[0] = Ordered(jumps);
		
		r1.starting = r1.states.len();
		r1.states.push(NFAState::new(new_transitions,false));
		return Ok((*r1).clone());
	}
}

impl From<&DFA> for NFA {
	fn from(dfa:&DFA) -> Self {
		let states:Vec<NFAState> = (&dfa.states).into_iter().map(|s| NFAState::from(s)).collect();
		return NFA::new(states,dfa.starting,dfa.alphabet.clone());
	}
}

impl From<&Regex> for NFA {
	fn from(reg:&Regex) -> Self {
		return match &reg.tree {
			None => NFA::get_never_accept(reg.alphabet.clone()),
			Some(tree) => tree.to_nfa(reg.alphabet.clone()).expect("This only fails if two generated alphabets are different, which indicates a programming error, not a user error")
		};
	}
}
impl TryFrom<Vec<String>> for NFA {
	type Error=String;
	fn try_from(lines:Vec<String>) -> Result<Self,Self::Error> {
		let num_lines = lines.len();
		if num_lines < 3 {
			return Err("Input file is too short".to_string())
		}
		let alphabet:String = match crate::get_alphabet(&lines[0]){
			Err(e) => return Err(e),
			Ok(am) => am
		};
		let alphabet_hm = crate::get_alphabet_hm(&lines[0]);
		let starting = match lines[1].parse::<StateNum>() {
			Ok(n) => {
				if n <= num_lines - 2 && n>0 {
					n-1
				} else {
					return Err("Starting number is not suitable".to_string())
				}
			},
			Err(_) => return Err("Starting is not a number".to_string())
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
