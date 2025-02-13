use std::collections::HashMap;
use crate::Rslt;
use crate::dfa::DFA;
use crate::dfa::DFAState;

#[derive(Clone)]
pub struct NFAState {
    transitions:Vec<u64>,
    accepting:bool
}

pub struct NFA {
    states:HashMap<u64, NFAState>,
    starting:u64,
    alphabet:String
}

impl NFA {

pub fn run(&self, input_word:Option<&str>, output_dfa:Option<&str>) -> Rslt {
    let is_word:bool;

    let word:&str;
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

    if !(is_output||is_word) {
	return Rslt::Notodo;
    }
    let result_dfa:DFA;

    match self.to_dfa() {
	Ok(d) => result_dfa = d,
	Err(e) => return Rslt::Err(e)
    }

    if is_output {
	match crate::print_lines_to_file(result_dfa.format(),dfa_output_address) {
	    Ok(()) => (),
	    Err(e) => return Rslt::Err(e)
	}
    }
    
    if is_word {
	return result_dfa.run(word);
    }
    return Rslt::Nop;
    
}

fn to_dfa(&self) -> Result<DFA,String> {
    let starting_state_set:u64 = equivalence(self.starting,&self.states);
    let alphabet = &self.alphabet;
    let states = &self.states;
    
    let dfa_states:Vec<DFAState> = Vec::new();
    let starting = 0;

    let mut visited:HashMap<u64,NFAState> = HashMap::new();
    let mut ordered_visited:Vec<u64> = Vec::new();
    let mut frontier:Vec<u64> = Vec::new();

    let mut old_to_new_identifier:HashMap<u64,usize> = HashMap::new();
    let mut count = 1;
    
    frontier.push(starting_state_set);

    let upperbound:usize = alphabet.len() + 1;
    
    while frontier.len() > 0 {
	let consider = frontier[0]; //the first state-set to be considered will be the starting state-set

	old_to_new_identifier.insert(consider, count);
	ordered_visited.push(consider);
	count = count + 1;
	
	let accepting = is_accepting(consider, &states);
	let mut transitions:Vec<u64> = Vec::new();
	for transition in 1..upperbound { //start at 1 as transition 0 is the jump option
	    let result = get_next_states(consider,transition,&states);
	    transitions.push(result);
	    if !(visited.contains_key(&result) || frontier.contains(&result)) {
		frontier.push(result);
	    }
	}
	visited.insert(consider, NFAState{
	    transitions,
	    accepting
	});
	frontier.remove(0);
    }
    let mut states = dfa_states;
    for i in 0..(count-1) {
	let mut transitions:Vec<usize> = Vec::new();
	let current_state = ordered_visited[i];
	for t in &visited[&current_state].transitions { //in this case, the first transition is the first non-empty element of the alphabet
	    transitions.push(old_to_new_identifier[&t]);

	}
	let accepting = visited[&current_state].accepting;
	states.push(DFAState{transitions,accepting});
    }

    let alphabet_map:HashMap<char,usize>;
    match crate::dfa::alphabet_to_alphabet_map(&alphabet) {
	Err(e) => return Err(e),
	Ok(am) => alphabet_map =am
    }
   
    return Ok(DFA{alphabet_map,starting,states});
}

}

/*
fn code_to_list(input:u64) -> Vec<u64> {
    let mut result = Vec::<u64>::new();

    let mut i = 1;
    for _ in 1..64 {
	if (input & i) != 0 {
	    result.push(1);
	}
	i = i << 1;
    }
    return result;
}
*/

pub fn nfa_option(lines:Vec<String>, input_word:Option<&str>, output_dfa:Option<&str>) -> Rslt {
    let nfa: NFA;
    match lines_to_nfa(lines) {
	Ok(n) => nfa = n,
	Err(e) => return Rslt::Err(e)
    }
    return nfa.run(input_word, output_dfa);
}

fn lines_to_nfa(lines:Vec<String>) -> Result<NFA, String> {
    if lines.len()<3 {
	return Err(format!("Input file is too short"));
    }
    
    let alphabet_string = &lines[0];
    let alphabet:Vec<char> = alphabet_string.chars().collect();
    let mut alphabet_hashmap = HashMap::<char,u32>::new();
    let mut i = 1;
    for c in &alphabet {
	alphabet_hashmap.insert(*c,i);
	i = i+1;
    }
//    println!("Alphabet is {alphabet_hashmap:?}");
    let mut nfa_states = HashMap::<u64, NFAState>::new();
    //this introduces a limit of 64 initial states, but that is fine for the small automata I'll be experimenting with.
    //will rework this system in a later update
    let start_state = lines[1].parse::<usize>().unwrap();
    let start_state: u64 = 1 << (start_state - 1);

    let mut i:u64 = 1;
    for state in &lines[2..] {
	let comma_split:Vec<&str> = state.split(",").collect();
	let num_parts = comma_split.len();

	let mut next_states:Vec<u64> = vec![0;alphabet.len() + 1];
	
	if num_parts > 1 {
	    for transition in (&comma_split[0..num_parts-1]).into_iter(){
		let parts:Vec<&str> = transition.split(":").collect();
//		println!{"{:?}",parts[0].chars()};
//		let letter:char = parts[0].chars().next().unwrap();

		let chars:Vec<char> = parts[0].chars().collect();
		let transition_num:usize = match chars.len() {
		    0 => 0,
		    1 => alphabet_hashmap[&chars[0]] as usize,
		    _ => return Err(format!("Each transition has one letter, not multiple"))
		};
		
		let value:u64 = parts[1].parse().unwrap();
		next_states[transition_num] = next_states[transition_num] | (1 << (value-1))
	    }
	}

	let accept:bool;
	match comma_split[num_parts - 1].parse() {
	    Ok(a) => accept = a,
	    Err(_) => return Err(format!("Poorly formatted accepting/not accepting value.")),
	}
	//println!("Adding state {}",i);
	nfa_states.insert(i, NFAState{
	    transitions:next_states,
	    accepting:accept
	});
	
	i = i << 1; //we can represent each individual state n as 2 ^(n-1), and a set of multiple states (including singleton sets) as binary ORs of these values.
    }

    return Ok(NFA {
	states: nfa_states,
	starting: start_state,
	alphabet: String::from(alphabet_string)
    });
}

fn get_next_states(stateset:u64, letter:usize, states:&HashMap<u64, NFAState>) -> u64 { //only use this on equivalence sets of another state set, so do not need to cover jumps from the given stateset

    if stateset == 0 {
	return 0;
    }
    
    let mut i:u64 = 1;
    let mut result:u64 = 0;
    
    for _ in 1..64 {
	if (stateset & i) != 0 {
	    result = result | equivalence(states[&i].transitions[letter as usize], &states);
	}
	i = i << 1;
    }

    return result;
}

fn is_accepting(stateset:u64, states:&HashMap<u64, NFAState>) -> bool { // stateset is an equivalence class, so we don't need to consider jumps

    if stateset == 0 {
	return false;
    }
    
    let mut i:u64 = 1;
    for _ in 1..64 {
	if (stateset & i) != 0 {
	    if states[&i].accepting == true {
		return true;
	    }
	}
	i = i << 1;
    }
    return false;
}
fn equivalence(code:u64, states:&HashMap<u64,NFAState>) -> u64 {

    //graph exploration so recursion is necessary
    fn recurse_eq(code:u64, states:&HashMap<u64,NFAState>, visited_in:u64) -> u64 {
	let mut visited = visited_in;
	let mut result = 0;
	
	let mut i:u64 = 1;
	for _ in 1..64 {
	    //identify the individual states in the given state set, as long as they've not been visited before
	    if (i & code != 0) && (i & visited == 0) {
		visited = visited | i;
		result = result | i;
		//println!("Looking for {}", i);
		let jump_state_set = states[&i].transitions[0];
		result = result | recurse_eq(jump_state_set,states,visited);
	    }

	    i = i << 1;
	}
	return result;
    }
    return recurse_eq(code,states,0);
}
