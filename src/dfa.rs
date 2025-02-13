use std::collections::HashMap;
use crate::Rslt;

#[derive(Clone)]
pub struct DFA {
    states: Vec<DFAState>,
    alphabet_map: HashMap<char,usize>,
    starting:usize	
}

impl DFA {
pub fn run(&self, word:&str) -> Rslt {
   
    let mut current_state=self.starting;
    for letter in word.chars() {

	if !self.alphabet_map.contains_key(&letter) {
	    return Rslt::Rej;//if a letter in the word is not in the alphabet, reject the word
	}	
	let equivalent = self.alphabet_map[&letter];
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
pub fn format(&self) -> Vec<String> {
    let mut output:Vec<String> = Vec::new();
    output.push(DFA::format_alphabet(&self.alphabet_map));
    output.push((self.starting+1).to_string());
    for state in &self.states {
	let mut line=String::new();
	for transition in &state.transitions {
	    line.push_str(&format!("{},",transition+1));
	}
	line.push_str(&state.accepting.to_string());
	output.push(line);
    }
    return output;
}

fn format_alphabet(alphabet_map:&HashMap<char,usize>) -> String {
    let mut chars:Vec<char> = vec!['a';alphabet_map.len()];
    for k in alphabet_map.keys() {
	chars[alphabet_map[k]] = *k;
    }
    return chars.into_iter().collect();
}


}

#[derive(Clone)]
pub struct DFAState {
    transitions: Vec<usize>,
    accepting:bool
}

pub fn dfa_option(lines:Vec<String>, input_word:Option<&str>) -> Rslt {
    
    let word:&str;
    if let Some(in_word) = input_word {
	word=in_word;
    } else {
	return Rslt::Notodo;
    }

    let dfa:DFA;
    match lines_to_dfa(lines) {
	Ok(d) => dfa = d,	
	Err(e) => return Rslt::Err(e)
    }
    return dfa.run(word);
}

fn lines_to_dfa(lines:Vec<String>) -> Result<DFA,String> {
    if lines.len()<3 {
	return Err(format!("Input file is too short"));
    }
    
    let alphabet = &lines[0];
    let alphabet_map;
    match alphabet_to_alphabet_map(alphabet) {
	Ok(am) => alphabet_map = am,
	Err(e) => return Err(e)
    }

    let starting = lines[1].parse::<usize>().unwrap()-1;   
    
    let num_states = lines.len()-2;
    let num_letters = alphabet.len();
    let mut states:Vec<DFAState> = Vec::new();
    
    for line in &lines[2..] {
	match line_to_dfa_state(line,num_letters,num_states) {
	    Err(e) => return Err(e),
	    Ok(new_state)=> states.push(new_state)
	}
    }
    return Ok(DFA{states,alphabet_map,starting});

}

pub fn alphabet_to_alphabet_map(alphabet:&str) -> Result<HashMap<char,usize>, String> {
    let mut alphabet_map:HashMap<char,usize> = HashMap::new();

    let mut i:usize = 0;
    for letter in alphabet.chars() {
	if alphabet_map.contains_key(&letter) {
	    return Err(format!("The alphabet contains a duplicate letter"));
	} else {
	    alphabet_map.insert(letter,i);
	    i = i + 1;
	}
    }
    return Ok(alphabet_map);
}

fn line_to_dfa_state(line:&String,num_letters:usize,num_states:usize) -> Result<DFAState,String> {
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
	DFAState{
	    transitions: next_states,
	    accepting: accept}
    );
}
