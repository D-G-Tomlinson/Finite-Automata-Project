use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use clap::Parser; //allows me flexibility with reading commandline arguments

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Location of the file storing the finite automaton
    #[arg(short,long)]
    input: Option<String>,

    /// The word to be analysed
    #[arg(short, long)]
    word: Option<String>,

    /// Location of the DFA file to write the converted input to
    #[arg(short, long)]
    dfa_output: Option<String>,

    /// Regular expression to be evaluated or converted. This option is not currently implemented
    #[arg(short, long)]
    regex: Option<String>,
}

struct DFAState {
    transitions: Vec<usize>,
    accepting:bool
}
#[derive(Debug, PartialEq, Eq)]
enum InputType{
    DFA,
    NFA,
    REGEX
}


//Although some input validation is done, it is not comprehensive - this project is an exercise in regular langauge representation, conversion and computation, rather than data validation
fn main() {

    println!("Use the --help option (i.e. cargo run -- --help) to learn about possible options.");
    
    let mut input_type = InputType::DFA; //doesn't really matter as long as it's not REGEX
    let cli = Cli::parse();

    let regex:&str;
    if let Some(in_regex) = cli.regex.as_deref() {
	regex = in_regex;
	input_type = InputType::REGEX
    }

    
    let address:&str;
    let mut contents = String::new();
    let mut lines = Vec::<&str>::new();
    if let Some(input) = cli.input.as_deref() {
		if input_type == InputType::REGEX {
			println!("No input file for regex operations.");
			return;
		} else {
			address=input;
			
			let file_type = address.split('.').last().unwrap().to_uppercase();
			match file_type.as_str() {
				"DFA" => input_type = InputType::DFA,
				"NFA" => input_type = InputType::NFA,
				_ => {
						println!("File type is unsupported.");
						return;
					}
			}
			

			match File::open(address) {
				Ok(mut f) => _ = f.read_to_string(&mut contents),
				Err(_) => {
					println!("Cannot read file.");
					return;
				}
			}
			lines = contents.lines().collect();
		}
    } else if input_type != InputType::REGEX {
	println!("No automata or regex provided.");
	return;
    }


    let result = match input_type {
	InputType::DFA => run_dfa(lines, cli.word.as_deref().map(|s| s.to_string())),
	InputType::NFA => run_nfa(lines,
				  cli.word.as_deref().map(|s| s.to_string()),
				  cli.dfa_output.as_deref().map(|s| s.to_string())
				  ),
	InputType::REGEX => Err(format!("Not implemented yet"))
    };
    
    println!("{}", match result {
	Err(e) => e,
	Ok(true) => format!("ACCEPT"),
	Ok(false) => format!("REJECT")
    });
    return;
}

fn run_dfa(lines:Vec<&str>, input_word:Option<String>) -> Result<bool,String> {
   
    if lines.len()<3 {
	return Err(format!("Input file is too short"));
    }
    let word:String;
    if let Some(in_word) = input_word {
	word=in_word;
    } else {
	return Err(format!("Nothing to do: No word provided."));
    }
    let word = word.as_str();
    
    let alphabet = lines[0];
    let mut alphabet_map:HashMap<char,usize> = HashMap::new();

    let mut i:usize = 0;
    for letter in alphabet.chars() {

	if letter == 'e' {
	    return Err(format!("To avoid confusion with the empty word, the letter e cannot be part of the alphabet"));
	}
	
	alphabet_map.insert(letter,i);
	i = i + 1;
    }

    let num_states = lines.len()-2;
    
    let mut current_state=lines[1].parse::<usize>().unwrap()-1;

    let num_letters = alphabet.len();
    let mut states:Vec<DFAState> = Vec::new();
    
    for state in &lines[2..] {
	let split_state:Vec<&str> = state.split(",").collect();

	if split_state.len() != num_letters+1 {
	    return Err(format!("Invalid number of elements on line"));
	}
	
	let mut next_states:Vec<usize> = Vec::new();
	for ns in (&split_state[0..num_letters]).into_iter(){
	    match ns.parse::<usize>() {
		Ok(v) => {
		    if v >= 1 && v <= num_states {
			next_states.push(v-1)
		    } else {
			return Err(format!("Value of next state is outside of the bounds of possible states"));
		    }
		},
		Err(_) => return Err(format!("Value for next state is not a valid number")),
	    }
	}//.map(|n| n.parse::<usize>().unwrap()-1).collect();

	let accept:bool;
	match split_state[num_letters].parse() {
	    Ok(a) => accept = a,
	    Err(_) => return Err(format!("Poorly formatted accepting/not accepting value.")),
	}
	
	let new_state = DFAState{
	    transitions: next_states,
	    accepting: accept
	}; 
	states.push(new_state);
    }

    for letter in word.chars() {

	if !alphabet_map.contains_key(&letter) {
	    return Err(format!("Letter in word is not in alphabet"));
	}	
	let equivalent = alphabet_map[&letter];
	let current_state_obj = &states[current_state];
	let edges = &current_state_obj.transitions;
	current_state = edges[equivalent];
    }
//    println!{"Final state is {}",current_state}
    return Ok(states[current_state].accepting)
    
}

#[derive(Clone)]
struct NFAState {
    transitions:Vec<u64>,
    accepting:bool
}

struct NFA {
    states:HashMap<u64, NFAState>,
    starting:u64,
    alphabet:String
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


fn run_nfa(lines:Vec<&str>, input_word:Option<String>, output_option:Option<String>) -> Result<bool,String> {
    if lines.len()<3 {
	return Err(format!("Input file is too short"));
    }
    let is_word:bool;

    if let Some(_) = input_word {
	is_word = true;
    } else {
	is_word = false;
    }
    
    let output;
    let is_output:bool;

    if let Some(in_output) = output_option {
	output = in_output;
	let file_type = output.split('.').last().unwrap().to_uppercase();
	if file_type != "DFA" {
	    return Err(format!("Can only write to .dfa files"));
	}
	is_output = true;
    } else {
	output = format!("");
	is_output = false;
    }

    if !(is_output||is_word) {
	return Err(format!("Nothing to do."));
    }

    let alphabet_string = lines[0];
    let alphabet:Vec<char> = alphabet_string.chars().collect();
    let mut alphabet_hashmap = HashMap::<char,u32>::new();
    let mut i = 1;
    for c in &alphabet {
	if *c != 'e' {
	    alphabet_hashmap.insert(*c,i);
	} else {
	    return Err(format!("To avoid confusion with the empty word, the letter e cannot be part of the alphabet"));
	}
	i = i+1;
    }
    alphabet_hashmap.insert('e',0);
    let mut nfa_states = HashMap::<u64, NFAState>::new();
    //this introduces a limit of 64 initial states, but that is fine for the small automata I'll be experimenting with.
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
		let letter:char = parts[0].chars().next().unwrap();
		let value:u64 = parts[1].parse().unwrap();
		next_states[alphabet_hashmap[&letter] as usize] = next_states[alphabet_hashmap[&letter] as usize] | (1 << (value-1))
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

    let initial_nfa = NFA {
	states: nfa_states,
	starting: start_state,
	alphabet: String::from(alphabet_string)
    };

    let result_dfa:Vec<String> = nfa_to_dfa(initial_nfa);

    if is_output {
	if let Ok(mut file_ptr) = File::create(output){
	    for i in 0..result_dfa.len() - 1 {
		writeln!(file_ptr, "{}", result_dfa[i]).expect("Problem writing to the file");
	    }
	    write!(file_ptr, "{}", result_dfa[result_dfa.len() - 1]).expect("Problem writing to the file");
	}

    }
    
    if is_word {
	let mut str_result_dfa: Vec<&str> = Vec::new();
	for i in 0..result_dfa.len() {
	    str_result_dfa.push(result_dfa[i].as_str());
	}
	return run_dfa(str_result_dfa, input_word);
    }
    return Err(format!("Finished without computation"));
    
}

fn nfa_to_dfa(input:NFA) -> Vec<String> {
    let starting_state_set:u64 = equivalence(input.starting,&input.states);
    let alphabet = input.alphabet;
    let states = input.states;
    
    let mut result:Vec<String> = Vec::new();
    result.push(alphabet.clone());
    result.push(format!("1"));

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
    for i in 0..(count-1) {
	let current_state = ordered_visited[i];
	let mut value = String::new();
	for t in &visited[&current_state].transitions { //in this case, the first transition is the first non-empty element of the alphabet
	    let temp_string = format!("{},",old_to_new_identifier[&t]);
	    value.push_str(&temp_string);
	}
	let temp_string = visited[&current_state].accepting.to_string();
	value.push_str(&temp_string);
	result.push(value);
    }
    return result
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
