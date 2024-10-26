use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Location of the file storing the DSA
    #[arg(short,long)]
    input: Option<String>,

    /// The word to be analysed
    #[arg(short, long)]
    word: Option<String>,

    /// Location of the file to write the converted input to
    #[arg(short, long)]
    output: Option<String>,

    /// Regular expression to be evaluated or converted
    #[arg(short, long)]
    regex: Option<String>,
}

struct DFA_State {
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
				  cli.output.as_deref().map(|s| s.to_string())
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
    let mut states:Vec<DFA_State> = Vec::new();
    
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
	
	let new_state = DFA_State{
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

struct NFA_State {
    transitions:Vec<u64>,
    accepting:bool
}
fn equivalence(code:u64, states:&Hashmap<u64,NFA_State>) -> u64 {
    let included_states = code_to_list(code).map(|n| states[n]);
    for state in included_states {
	for s in self.transistions[0] {
	    
	}
    }
}}

struct NFA {
    states:Hashmap<u64, NFA_State>,
    starting:u64
}

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

fn run_nfa(lines:Vec<&str>, input_word:Option<String>, output_option:Option<String>) -> Result<bool,String> {
    if lines.len()<3 {
	return Err(format!("Input file is too short"));
    }
    let word;
    let is_word:bool;

    if let Some(in_word) = input_word {
	word = in_word;
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
	is_output = false;
    }

    if !(is_output||is_word) {
	return Err(format!("Nothing to do."));
    }

    let alphabet:&str = lines[0]
    let mut alphabet_hashmap = HashMap::<char,u32>::new();
    let mut i = 1;
    for c:char in alphabet.chars() {
	if c != 'e' {
	    alphabet_hashmap.insert(c,i);
	} else {
	    return Err(format!("To avoid confusion with the empty word, the letter e cannot be part of the alphabet"));
	}
    }

    let mut nfa_states = HashMap::<u64, NFA_State>::new();
    
    return Err(format!("Unfinished"));
}

fn get_combo_value(input:Vec<u32>
