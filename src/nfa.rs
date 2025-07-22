use std::fmt;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::Rslt;

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
					alphabet[&chars[0]] as usize
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
			Err(_)  => return Err(format!("Poorly formatted accepting value"))
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
			output.push(';');
		}		
		output.push_str(&self.accepting.to_string());
		return output;
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
	pub fn from_lines(lines:Vec<String>) -> Result<NFA,String> {
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
	
	fn get_equivalents(&self) -> Vec<Vec<usize>> {		
		let num_states = self.states.len();
		let mut eqs:Vec<Vec<usize>> = (0..num_states).map(|i| NFA::ordered_union(&vec![i],&self.states[i].transitions[0])).collect();
		let mut changed = true;
		while changed {
			changed = false;
			for i in 0..num_states {
				for j in 0..num_states {
					if eqs[i].contains(&j)  {
						let v1 = &eqs[i];
						let v2 = &eqs[j];
						let new = NFA::ordered_union(&v1,&v2);
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

	fn get_equivalents_vec(states:&Vec<usize>,equivalents:&Vec<Vec<usize>>) -> Vec<usize> {
		let mut result = Vec::new();
		for state in states {
			let eqs = &equivalents[*state];
			result = NFA::ordered_union(&result,eqs);
		}
		return result;
	}

	fn get_to(&self, from:usize, by:usize,equivalents:&Vec<Vec<usize>>) -> Vec<usize>{
		let mut result:Vec<usize> = Vec::new();
		let from_states = &equivalents[from];
		for state in from_states {
			let next = &self.states[*state].transitions[by];
			let next_states =NFA::get_equivalents_vec(next,equivalents);
			result = NFA::ordered_union(&result, &next_states);
		}
		return result;
	}

	fn get_to_vec(&self, from:&Vec<usize>, by:usize, equivalents:&Vec<Vec<usize>>) -> Vec<usize>{
		let eqs = from.into_iter().map(|state| self.get_to(*state,by,equivalents));
		let mut result:Vec<usize> = Vec::new();
		for state in eqs {
			result = NFA::ordered_union(&result,&state);
		}
		return result;
	}

	fn add_line_to_table(&self,new_states:&mut HashMap<Vec<usize>,usize>,frontier:&mut VecDeque<Vec<usize>>,state_table:&mut Vec<Vec<usize>>,accepts:&mut Vec<bool>,state:&Vec<usize>,equivalents:&Vec<Vec<usize>>) {
		new_states.insert(state.to_vec(),state_table.len());
		frontier.push_back(state.to_vec());
		accepts.push(self.is_accepting_vec(state.to_vec(),equivalents));
		state_table.push(Vec::new());
	}

	fn is_accepting(&self,input_state:&usize,equivalents:&Vec<Vec<usize>>) -> bool {
		let eqs = &equivalents[*input_state];
		for state in eqs {
			if self.states[*state].accepting {
				return true;
			}
		}
		return false;
	}

	fn is_accepting_vec(&self,input_states:Vec<usize>,equivalents:&Vec<Vec<usize>>) -> bool {
		for state in input_states {
			if self.is_accepting(&state,equivalents) {
				return true
			}
		}
		return false;
	}
	fn to_dfa(&self) -> DFA {
		let equivalents = self.get_equivalents();
		let mut new_states:HashMap<Vec<usize>,usize> = HashMap::new();
		let mut frontier:VecDeque<Vec<usize>> = VecDeque::new();
		let mut state_table:Vec<Vec<usize>> = Vec::new();
		let mut accepts:Vec<bool> = Vec::new();

		let num_letters = self.states[0].transitions.len()-1;
		
		let first_state = &equivalents[self.starting];
		self.add_line_to_table(&mut new_states,&mut frontier,&mut state_table,&mut accepts,&first_state,&equivalents);
		while !frontier.is_empty() {
			let current = NFA::get_equivalents_vec(&frontier.pop_front().unwrap(),&equivalents);
			let current_row = new_states[&current];			
			for i in 1..num_letters {
				let next = self.get_to_vec(&current,i,&equivalents);
				if !new_states.contains_key(&next) {
					self.add_line_to_table(&mut new_states,&mut frontier,&mut state_table,&mut accepts,&next,&equivalents);
				}
				state_table[current_row].push(new_states[&next]);
			}
		}

		let states:Vec<DFAState> = (0..state_table.len()).map(|i|DFAState::new(state_table[i].clone(),accepts[i])).collect();
		let starting = 0;
		
		return DFA::new(states,self.alphabet.clone(),starting);
	}
	
	pub fn run(&self,input_word:Option<&str>, output_dfa:Option<&str>) -> Rslt {
		let word:&str;
		let is_word:bool;
		if let Some(in_word) = input_word {
			is_word = true;
			word=in_word;
		} else {
			is_word = false;
			word="";
		}
			
		let dfa_output_address;
		let is_output:bool;
		if let Some(in_output) = output_dfa {
			dfa_output_address = in_output;
			let file_type = dfa_output_address.split('.').last().unwrap().to_uppercase();
			if file_type != "DFA" {
				return Rslt::Err(format!("Can only write to .dfa files"));
			}
			is_output = true;
		} else {
			dfa_output_address = "";
			is_output = false;
		}
				
		if !(is_word || is_output) {
			return Rslt::Notodo;
		}

		let dfa = self.to_dfa();
		
		if is_output {
			match crate::print_to_file(dfa.to_string(),dfa_output_address) {
				Ok(()) => (),
				Err(e) => return Rslt::Err(e)
			}
		}
		if is_word {
			return dfa.run(word);
		} else {
			return Rslt::Nop;
		}
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

pub fn nfa_option(lines:Vec<String>, input_word:Option<&str>, output_dfa:Option<&str>) -> Rslt {
    let nfa: NFA;
    match NFA::from_lines(lines) {
		Ok(n) => nfa = n,
		Err(e) => return Rslt::Err(e)
    }
    return nfa.run(input_word, output_dfa);
}

