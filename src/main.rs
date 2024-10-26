use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use clap::Parser;

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

    /// Regular expression to be evaluated or converted
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

fn main() {

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
	println!("No word provided.");
	return Err(format!("Nothing to do"));
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

struct NFAState {
    transitions:Vec<u64>,
    accepting:bool
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
		let jump_state_set = states[&i].transitions[0];
		result = result | recurse_eq(jump_state_set,states,visited);
	    }

	    i = i << 1;
	}
	return result;
    }
    return recurse_eq(code,states,0);
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

    let mut nfa_states = HashMap::<u64, NFAState>::new();
    let start_state = lines[1].parse::<usize>().unwrap();
    let start_state: u64 = 1 << (start_state - 1);

    let mut i:u64 = 1;
    for state in &lines[2..] {
	let comma_split:Vec<&str> = state.split(",").collect();
	let num_parts = comma_split.len();

	let mut next_states:Vec<u64> = vec![0;alphabet.len()];
	
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

	nfa_states.insert(i, NFAState{
	    transitions:next_states,
	    accepting:accept
	});
	
	i = i << 2; //we can represent each individual state n as 2 ^(n-1), and a set of multiple states (including singleton sets) as sums of these values.
    }

    let initial_nfa = NFA {
	states: nfa_states,
	starting: start_state,
	alphabet: String::from(alphabet_string)
    };

    let result_dfa:Vec<String> = nfa_to_dfa(initial_nfa);
    if is_word {
	let mut str_result_dfa: Vec<&str> = Vec::new();
	for i in 0..result_dfa.len() {
	    str_result_dfa.push(result_dfa[i].as_str());
	}
	return run_dfa(str_result_dfa, input_word);
    }

    if is_output {
	if let Ok(mut file_ptr) = File::create(output){
	    for i in 0..result_dfa.len() {
		writeln!(file_ptr, "{}", result_dfa[i]).expect("Problem writing to the file");
	    }
	    write!(file_ptr, "{}", result_dfa[result_dfa.len() - 1]).expect("Problem writing to the file");
	}
    }
    
    return Err(format!("Unfinished"));
}

fn nfa_to_dfa(input:NFA) -> Vec<String> {
    let mut result:Vec<String> = Vec::new();


    return result;
}

