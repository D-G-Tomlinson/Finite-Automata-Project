use std::collections::HashMap;
use crate::Rslt;
use crate::dfa::DFA;
use crate::dfa::DFAState;

#[derive(Clone)]
pub struct NFAState {
    transitions:Vec<Vec<usize>>,
    accepting:bool,
	equivalents:Option<Vec<usize>>,
}

impl NFAState {
    pub fn new(transitions:Vec<Vec<usize>>,accepting:bool,equivalents:Option<Vec<usize>>) -> NFAState {
		return NFAState{transitions,accepting,equivalents};
    }
	pub fn from_line(line:&String,alphabet:&HashMap<char,usize>,num_states:usize) -> Result<NFAState,String> {
		let comma_split:Vec<&str> = line.split(",").collect();
		let num_parts = comma_split.len();
		let mut transitions:Vec<Vec<usize>> = vec![Vec::new();alphabet.len()+1]; // +1 because of the empty letter, the jump, is not in the alphabet but does have transitions

		if num_parts > 1 {
			for transition in (&comma_split[0..num_parts-1]).into_iter() {
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
							alphabet[&chars[0]] as usize
						} else {
							return Err(format!("The transition letter must be in the alphabet"));
						}
					},
					_ => return Err(format!("Each transition must have one or no letters, not multple"))
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
				if !transitions[transition_num].contains(&value) {
					transitions[transition_num].push(value);
				}
			}
		}
		let accepting:bool = match comma_split[num_parts -1].parse() {
			Ok(a) => a,
			Err(_)  => return Err(format!("Poorly formatted accepting value"))
			
		};
		transitions.sort();
		return Ok(NFAState::new(transitions,accepting,None));
	}

	
}

pub struct NFA {
	states:Vec<NFAState>,
	starting:usize,
	alphabet:HashMap<char,usize>
}

impl NFA {
	pub fn new(states:Vec<NFAState>,starting:usize,alphabet:HashMap<char,usize>) -> NFA {
		return NFA{states, starting, alphabet};
	}
	pub fn from_lines(lines:Vec<String>) -> Result<NFA,String> {
		let num_lines = lines.len();
		if num_lines < 3 {
			return Err(format!("Input file is too short"))
		}
		let alphabet = NFA::get_alphabet_hm(&lines[0]);

		let starting = match lines[1].parse::<usize>() {
			Ok(n) => {
				if n <= num_lines - 2 && n>0 {
					n
				} else {
					return Err(format!("Starting number is not suitable"))
				}
			},
			Err(_) => return Err(format!("Starting is not a number"))
		};
		let state_lines = &lines[2..];
		let mut states:Vec<NFAState> = Vec::new();

		for line in state_lines{
			states.push(match NFAState::from_line(line,&alphabet,num_lines-2) {
				Ok(nfastate) => nfastate,
				Err(e) => return Err(e)
			});
	}
		return Ok(NFA::new(states,starting,alphabet));
	}

	fn get_alphabet_hm(alphabet:&str) -> HashMap<char,usize> {
		let alphabet:Vec<char> = alphabet.chars().collect();
		let mut alphabet_hashmap = HashMap::<char,usize>::new();
		let mut i = 1; //0 represents the jump, which is the empty letter
		for c in &alphabet {
			if !alphabet_hashmap.contains_key(c) {
				alphabet_hashmap.insert(*c,i);
				i = i + 1;
			}
		}
		return alphabet_hashmap;
	}
/*
	fn to_dfa(&self) -> DFA {
		
	}
	 */
	fn ordered_union(v1:&Vec<usize>,v2:&Vec<usize>) -> Vec<usize> {
		if v1.len()==0 {
			return v2.to_vec();
		} else if v2.len()==0 {
			return v1.to_vec();
		}
		
		let mut result:Vec<usize> = Vec::new();
		let mut i = 0;
		let mut j = 0;
		if v1[0]<v2[0] {
			result.push(v1[0]);
		} else {
			result.push(v2[0]);
		}
		
		while i < v1.len() || j < v2.len() {
			let consider_v1:bool;
			if i==v1.len() {
				consider_v1 = false;
			} else if j==v2.len() {
				consider_v1 = true;
			} else if v1[i]<=v2[j] {
				consider_v1 = true;
			} else {
				consider_v1 = false;
			}
			if consider_v1 {
				if result.last().unwrap() == &v1[i] {
					i=i+1
				} else {
					result.push(v1[i]);
					i=i+1;
				}
			} else {
				if result.last().unwrap() == &v2[j] {
					j=j+1
				} else {
					result.push(v2[j]);
					j=j+1;
				}
			}
		}
		return result;
	}
	
	fn check_all_vec(input:&Vec<bool>) -> bool {
		for i in input {
			if !i {
				return false;
			}
		}
		return true;
	}
	fn get_equivalents(&mut self) {
		let num_states = self.states.len();
		let mut eqs:Vec<Vec<usize>> = (0..num_states).map(|i| NFA::ordered_union(&vec![i],&self.states[i].transitions[0])).collect();
		let mut changed =vec![true;num_states];
		while !NFA::check_all_vec(&changed) {
			for i in 0..num_states {
				changed[i]=false;
				for j in 0..num_states {
					if changed[i] {
						let v1 = &eqs[i];
						let v2 = &eqs[j];
						let new = NFA::ordered_union(&v1,&v2);
						if v1 == &new {
							changed[i]=true;
							eqs[i]=new;
						}
					}
				}
			}
		}
		for i in 0..num_states {
			let old = &self.states[i];
			let transitions = &old.transitions;
			let accepting = old.accepting;
			let equivalents = Some(eqs[i].clone());
			self.states[i] = NFAState::new(transitions.to_vec(),accepting,equivalents);
		}
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

