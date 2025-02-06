use std::collections::HashMap;
use crate::Rslt;

struct DFA {
    states: Vec<DFAState>,
    alphabet_map: HashMap<char,usize>,
    starting:usize
}

struct DFAState {
    transitions: Vec<usize>,
    accepting:bool
}

pub fn dfa_option(lines:Vec<String>, input_word:Option<&str>) -> Rslt {
    let dfa:DFA;
    match lines_to_dfa(lines) {
	Err(e) => return Rslt::Err(e),
	Ok(d) => dfa = d
    }

    return run_dfa(dfa,input_word);
}

fn lines_to_dfa(lines:Vec<String>) -> Result<DFA,String> {
    if lines.len()<3 {
	return Err(format!("Input file is too short"));
    }
    
    let alphabet = &lines[0];
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
pub fn run_dfa(dfa:DFA, input_word:Option<&str>) -> Rslt {
   
    let word:&str;
    if let Some(in_word) = input_word {
	word=in_word;
    } else {
	return Rslt::Notodo;
    }

    let mut current_state=dfa.starting;
    for letter in word.chars() {

	if !dfa.alphabet_map.contains_key(&letter) {
	    return Rslt::Rej;//if a letter in the word is not in the alphabet, reject the word
	}	
	let equivalent = dfa.alphabet_map[&letter];
	let current_state_obj = &dfa.states[current_state];
	let edges = &current_state_obj.transitions;
	current_state = edges[equivalent];
    }
//    println!{"Final state is {}",current_state}
    return match dfa.states[current_state].accepting {
	true => Rslt::Acc,
	false => Rslt::Rej
    }
    
}
