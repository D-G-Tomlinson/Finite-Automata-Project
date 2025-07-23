use crate::Rslt;
use std::fmt;

use crate::nfa::NFA;
use crate::nfa::NFAState;

#[derive(Clone)]
pub struct DFA {
    states: Vec<DFAState>,
    alphabet:String,
    starting:usize	
}

impl DFA {
    pub fn new(states:Vec<DFAState>, alphabet:String,starting:usize) -> DFA {
		DFA{states, alphabet, starting}
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
    
	fn from_lines(lines:Vec<String>) -> Result<DFA,String> {
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
		return Ok(DFA::new(states,alphabet,starting));
		
	}
	pub fn to_nfa(&self) -> NFA {
		let states:Vec<NFAState> = (&self.states).into_iter().map(|s| s.to_nfastate()).collect();
		return NFA::new(states,self.starting,self.alphabet.clone());
	}
	
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
    transitions: Vec<usize>,
    accepting:bool
}

impl DFAState {
    pub fn new(transitions:Vec<usize>,accepting:bool) -> DFAState {
		DFAState{transitions, accepting}
    }
	fn from_line(line:&String,num_letters:usize,num_states:usize) -> Result<DFAState,String> {
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
			DFAState::new(
				next_states,
				accept)
		);
	}

	pub fn to_nfastate(&self) -> NFAState {
		let mut transitions:Vec<Vec<usize>> = Vec::new();
		transitions.push(Vec::new());
		for next in &self.transitions {
			transitions.push(vec![*next]);
		}
		return NFAState::new(transitions,self.accepting);
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
