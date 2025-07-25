use crate::Rslt;
use std::fmt;

use std::collections::HashMap;
use std::collections::VecDeque;

use crate::nfa::NFA;
use crate::nfa::NFAState;

use std::convert::From;
use std::convert::TryFrom;

#[derive(Clone)]
pub struct DFA {
    pub states: Vec<DFAState>,
    pub alphabet:String,
    pub starting:usize	
}

impl DFA {
    pub fn new(states:Vec<DFAState>, alphabet:String,starting:usize) -> Self {
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
			current_state = edges[equivalent];
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
			return Err(format!("Input file is too short"));
		}
		
		let alphabet = match crate::get_alphabet(&lines[0]) {
			Err(e) => return Err(e),
			Ok(ab) => ab
		};
		let starting = lines[1].parse::<usize>().unwrap()-1;   
		
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
		let equivalents = get_equivalents(&nfa.states);
		let mut new_states:HashMap<Vec<usize>,usize> = HashMap::new();
		let mut frontier:VecDeque<Vec<usize>> = VecDeque::new();
		let mut state_table:Vec<Vec<usize>> = Vec::new();
		let mut accepts:Vec<bool> = Vec::new();

		let num_letters = nfa.states[0].transitions.len()-1;
		
		let first_state = &equivalents[nfa.starting];
		add_line_to_table(&nfa.states,&mut new_states,&mut frontier,&mut state_table,&mut accepts,&first_state,&equivalents);
		while !frontier.is_empty() {
			let current = get_equivalents_vec(&frontier.pop_front().unwrap(),&equivalents);
			let current_row = new_states[&current];			
			for i in 1..(num_letters+1) {
				let next = get_to_vec(&nfa.states,&current,i,&equivalents);
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


fn get_to(states:&Vec<NFAState>, from:usize, by:usize,equivalents:&Vec<Vec<usize>>) -> Vec<usize>{
	let mut result:Vec<usize> = Vec::new();
	let from_states = &equivalents[from];
	for state in from_states {
		let next = &states[*state].transitions[by];
		let next_states =get_equivalents_vec(next,equivalents);
		result = ordered_union(&result, &next_states);
	}
	return result;
}

fn get_to_vec(states:&Vec<NFAState>, from:&Vec<usize>, by:usize, equivalents:&Vec<Vec<usize>>) -> Vec<usize>{
	let eqs = from.into_iter().map(|state| get_to(states,*state,by,equivalents));
	let mut result:Vec<usize> = Vec::new();
	for state in eqs {
		result = ordered_union(&result,&state);
		}
	return result;
}
	

fn get_equivalents_vec(states:&Vec<usize>,equivalents:&Vec<Vec<usize>>) -> Vec<usize> {
	let mut result = Vec::new();
	for state in states {
		let eqs = &equivalents[*state];
		result = ordered_union(&result,eqs);
	}
	return result;
}


fn get_equivalents(states:&Vec<NFAState>) -> Vec<Vec<usize>> {		
	let num_states = states.len();
	let mut eqs:Vec<Vec<usize>> = (0..num_states).map(|i| ordered_union(&vec![i],&states[i].transitions[0])).collect();
	let mut changed = true;
	while changed {
		changed = false;
		for i in 0..num_states {
			for j in 0..num_states {
				if eqs[i].contains(&j)  {
					let v1 = &eqs[i];
					let v2 = &eqs[j];
					let new = ordered_union(&v1,&v2);
					if v1 != &new {
						changed=true;
						eqs[i]=new;
					}
				}
			}
		}
	}
	return eqs;
}	

	fn add_line_to_table(states:&Vec<NFAState>,new_states:&mut HashMap<Vec<usize>,usize>,frontier:&mut VecDeque<Vec<usize>>,state_table:&mut Vec<Vec<usize>>,accepts:&mut Vec<bool>,state:&Vec<usize>,equivalents:&Vec<Vec<usize>>) {
		new_states.insert(state.to_vec(),state_table.len());
		frontier.push_back(state.to_vec());
		accepts.push(is_accepting_vec(states,state.to_vec(),equivalents));
		state_table.push(Vec::new());
	}

	fn is_accepting(states:&Vec<NFAState>,input_state:&usize,equivalents:&Vec<Vec<usize>>) -> bool {
		let eqs = &equivalents[*input_state];
		for state in eqs {
			if states[*state].accepting {
				return true;
			}
		}
		return false;
	}

	fn is_accepting_vec(states:&Vec<NFAState>,input_states:Vec<usize>,equivalents:&Vec<Vec<usize>>) -> bool {
		for state in input_states {
			if is_accepting(states,&state,equivalents) {
				return true
			}
		}
		return false;
	}

	
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
    pub transitions: Vec<usize>,
    pub accepting:bool
}

impl DFAState {
    pub fn new(transitions:Vec<usize>,accepting:bool) -> Self {
		Self{transitions, accepting}
    }
	fn from_line(line:&String,num_letters:usize,num_states:usize) -> Result<Self,String> {
		let split_state:Vec<&str> = line.split(",").collect();
		if split_state.len() != num_letters+1 {
			return Err(format!("Invalid number of elements on line"));
		}
		
		let mut next_states:Vec<usize> = Vec::new();
		for next_state_str in (&split_state[0..num_letters]).into_iter(){
			match next_state_str.parse::<usize>() {
				Ok(next_state_num) => {
					if next_state_num >= 1 && next_state_num <= num_states {
						next_states.push(next_state_num-1)
					} else {
						return Err(format!("Value of next state is outside of the bounds of possible states"));
					}
				},
				Err(_) => return Err(format!("Value for next state is not a valid number")),
			}
		}
		
		let accept:bool;
		match split_state[num_letters].parse() {
			Ok(a) => accept = a,
			Err(_) => return Err(format!("Poorly formatted accepting/not accepting value.")),
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
