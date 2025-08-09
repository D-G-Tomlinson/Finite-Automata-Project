use crate::StateNum;
use crate::Ordered;

use crate::Rslt;
use std::fmt;

use std::collections::HashMap;
use std::collections::VecDeque;

use crate::nfa::NFA;
use crate::nfa::NFAState;

use std::convert::From;
use std::convert::TryFrom;

use crate::Index1;

#[derive(Clone)]
pub struct DFA {
    pub states: Vec<DFAState>,
    pub alphabet:String,
    pub starting:StateNum	
}

impl DFA {
    pub fn new(states:Vec<DFAState>, alphabet:String,starting:StateNum) -> Self {
		Self{states, alphabet, starting}
    }
    
    pub fn run(&self, word:&str) -> Rslt {
		let alphabet_map = crate::get_alphabet_hm(&self.alphabet);
		let mut current_state=self.starting;
		for letter in word.chars() {
			if !alphabet_map.contains_key(&letter) {
				return Rslt::Rej;//if a letter in the word is not in the alphabet, reject the word
			}	
			let equivalent = alphabet_map[&letter];
			let current_state_obj = &self.states[current_state];
			let edges = &current_state_obj.transitions;
			current_state = edges[equivalent.0];
		}
		//    println!{"Final state is {}",current_state}
		return match self.states[current_state].accepting {
			true => Rslt::Acc,
			false => Rslt::Rej
		}
		
    }
    
}

impl TryFrom<Vec<String>> for DFA {
	type Error=String;
	fn try_from(lines:Vec<String>) -> Result<Self,Self::Error> {
		if lines.len()<3 {
			return Err("Input file is too short".to_string());
		}
		
		let alphabet = match crate::get_alphabet(&lines[0]) {
			Err(e) => return Err(e),
			Ok(ab) => ab
		};
		let starting = lines[1].parse::<StateNum>().unwrap()-1;   
		
		let num_states = lines.len()-2;
		let num_letters = alphabet.len();
		let mut states:Vec<DFAState> = Vec::new();
		
		for line in &lines[2..] {
			match DFAState::from_line(line,num_letters,num_states) {
				Err(e) => return Err(e),
				Ok(new_state)=> states.push(new_state)
			}
		}
		return Ok(Self::new(states,alphabet,starting));
		
	}
}

impl From<&NFA> for DFA {
	fn from(nfa:&NFA) -> Self {
		let equivalents:Vec<Ordered> = get_equivalents(&nfa.states);
		let mut new_states:HashMap<Ordered,StateNum> = HashMap::new();
		let mut frontier:VecDeque<Ordered> = VecDeque::new();
		let mut state_table:Vec<Vec<StateNum>> = Vec::new();
		let mut accepts:Vec<bool> = Vec::new();

		let num_letters = nfa.states[0].transitions.len()-1;
		
		let first_state = &equivalents[nfa.starting];
		add_line_to_table(&nfa.states,&mut new_states,&mut frontier,&mut state_table,&mut accepts,first_state,&equivalents);
		while !frontier.is_empty() {
			let current = get_equivalents_vec(&frontier.pop_front().unwrap().0,&equivalents);
			let current_row = new_states[&current];			
			for i in 1..(num_letters+1) {
				let i = Index1(i);
				let next = get_to_vec(&nfa.states,&current.0,i,&equivalents);
				if !new_states.contains_key(&next) {
					add_line_to_table(&nfa.states,&mut new_states,&mut frontier,&mut state_table,&mut accepts,&next,&equivalents);
				}
				state_table[current_row].push(new_states[&next]);
			}
		}

		let states:Vec<DFAState> = (0..state_table.len()).map(|i|DFAState::new(state_table[i].clone(),accepts[i])).collect();
		let starting = 0;
		
		return DFA::new(states,nfa.alphabet.clone(),starting);
	}
}


fn get_to(states:&Vec<NFAState>, from:StateNum, by:Index1,equivalents:&Vec<Ordered>) -> Ordered {
	let mut result:Ordered = Ordered(Vec::new());
	let Ordered(from_states) = &equivalents[from];
	for state in from_states {
		let next = &states[*state].transitions[by.0].0;
		let next_states = get_equivalents_vec(next,equivalents);
		result = result.join(&next_states);
	}
	return result;
}

fn get_to_vec(states:&Vec<NFAState>, from:&Vec<StateNum>, by:Index1, equivalents:&Vec<Ordered>) -> Ordered {
	let eqs = from.into_iter().map(|state| get_to(states,*state,by,equivalents));
	let mut result:Ordered = Ordered(Vec::new());
	for state in eqs {
		result = result.join(&state);
		}
	return result;
}
	

fn get_equivalents_vec(states:&Vec<StateNum>,equivalents:&Vec<Ordered>) -> Ordered {
	let mut result = Ordered(Vec::new());
	for state in states {
		let eqs = &equivalents[*state];
		result = result.join(eqs);
	}
	return result;
}


fn get_equivalents(states:&Vec<NFAState>) -> Vec<Ordered> {		
	let num_states = states.len();
	let mut eqs:Vec<Ordered> = (0..num_states)
		.map(|i| Ordered(vec![i]).join(&states[i].transitions[0])).collect();
	let mut changed = true;
	while changed {
		changed = false;
		for i in 0..num_states {
			for j in 0..num_states {
				if eqs[i].0.contains(&j)  {
					let v1 = &eqs[i];
					let v2 = &eqs[j];
					let new = v1.join(&v2);
					if v1.0 != new.0 {
						changed=true;
						eqs[i]=new;
					}
				}
			}
		}
	}
	return eqs;
}	

fn add_line_to_table(nfa_states:&Vec<NFAState>,
					 new_states:&mut HashMap<Ordered,StateNum>,
					 frontier:&mut VecDeque<Ordered>,
					 state_table:&mut Vec<Vec<StateNum>>,
					 accepts:&mut Vec<bool>,
					 state:&Ordered,
					 equivalents:&Vec<Ordered>) {
	new_states.insert(state.clone(),state_table.len());
	frontier.push_back(state.clone());
	accepts.push(is_accepting_vec(nfa_states,&state,equivalents));
	state_table.push(Vec::new());
}

fn is_accepting(nfa_states:&Vec<NFAState>,input_state:&StateNum,equivalents:&Vec<Ordered>) -> bool {
	let Ordered(eqs) = &equivalents[*input_state];
	for state in eqs {
		if nfa_states[*state].accepting {
				return true;
		}
	}
	return false;
}

fn is_accepting_vec(nfa_states:&Vec<NFAState>,input_states:&Ordered,equivalents:&Vec<Ordered>) -> bool {
	for state in &input_states.0 {
		if is_accepting(nfa_states,&state,equivalents) {
			return true
		}
	}
	return false;
}



impl fmt::Display for DFA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut output:String = String::new();
		output.push_str(&self.alphabet);
		output.push('\n');
		output.push_str(&(self.starting+1).to_string());
		for state in &self.states {
			output.push('\n');
			output.push_str(&state.to_string());
		}
		write!(f, "{}",output)
    }
}

#[derive(Clone)]
pub struct DFAState {
    pub transitions: Vec<StateNum>,
    pub accepting:bool
}

impl DFAState {
    pub fn new(transitions:Vec<StateNum>,accepting:bool) -> Self {
		Self{transitions, accepting}
    }
	fn from_line(line:&String,num_letters:usize,max_state:StateNum) -> Result<Self,String> {
		let split_state:Vec<&str> = line.split(",").collect();
		if split_state.len() != num_letters + 1 {
			return Err("Invalid number of elements on line".to_string());
		}
		
		let mut next_states:Vec<StateNum> = Vec::new();
		for next_state_str in (&split_state[0..num_letters]).into_iter(){
			match next_state_str.parse::<StateNum>() {
				Ok(next_state_num) => {
					if next_state_num >= 1 && next_state_num <= max_state {
						next_states.push(next_state_num-1)
					} else {
						return Err("Value of next state is outside of the bounds of possible states".to_string());
					}
				},
				Err(_) => return Err("Value for next state is not a valid number".to_string()),
			}
		}
		
		let accept:bool;
		match split_state[num_letters].parse() {
			Ok(a) => accept = a,
			Err(_) => return Err("Poorly formatted accepting/not accepting value.".to_string()),
		}
		
		return Ok(
			Self::new(
				next_states,
				accept)
		);
	}
}
impl fmt::Display for DFAState {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut output:String=String::new();
		for transition in &self.transitions {
			output.push_str(&(transition+1).to_string());
			output.push(',');
		}
		output.push_str(&self.accepting.to_string());
		write!(f, "{}", output)
	}
}
