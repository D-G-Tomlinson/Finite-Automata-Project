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
    #[arg(short, long, value_name = "FILE")]
    word: Option<String>,
}

struct State {
    transitions: Vec<usize>,
    accepting:bool
}

fn main() {
    let cli = Cli::parse();

    let address:&str;
    if let Some(input) = cli.input.as_deref() {
	address=input;
    } else {
	println!("No DFA provided.");
	return;
    }
    let mut contents = String::new();
    match File::open(address) {
	Ok(mut f) => _ = f.read_to_string(&mut contents),
	Err(_) => {
	    println!("Cannot read file.");
	    return;
	}
    }
    let word:&str;
    if let Some(in_word) = cli.word.as_deref() {
	word=in_word;
    } else {
	println!("No word provided.");
	return;
    }
    let lines:Vec<&str> = contents.lines().collect();

  
    println!{"{}", match run_dfa(lines, word) {
	None => "Error with the automaton or the word.",
	Some(true) => "ACCEPT",
	Some(false) => "REJECT"
    }};
    return;
}

fn run_dfa(lines:Vec<&str>, word:&str) -> Option<bool> {
    let alphabet = lines[0];
    let mut alphabet_map:HashMap<char,usize> = HashMap::new();

    let mut i:usize = 0;
    for letter in alphabet.chars() {
	alphabet_map.insert(letter,i);
	i = i + 1;
    }

    let num_states = lines.len()-2;
    
    let mut current_state=lines[1].parse::<usize>().unwrap()-1;

    let num_letters = alphabet.len();
    let mut states:Vec<State> = Vec::new();
    
    for state in &lines[2..] {
	let split_state:Vec<&str> = state.split(",").collect();

	if split_state.len() != num_letters+1 {
	    return None;
	}
	
	let mut next_states:Vec<usize> = Vec::new();
	for ns in (&split_state[0..num_letters]).into_iter(){
	    match ns.parse::<usize>() {
		Ok(v) => {
		    if v > 0 && v <= num_states {
			next_states.push(v-1)
		    } else {
			return None
		    }
		},
		Err(_) => return None
	    }
	}//.map(|n| n.parse::<usize>().unwrap()-1).collect();

	let accept:bool;
	match split_state[num_letters].parse() {
	    Ok(a) => accept = a,
	    Err(_) => return None
	}
	
	let new_state = State{
	    transitions: next_states,
	    accepting: accept
	}; 
	states.push(new_state);
    }

    for letter in word.chars() {

	if !alphabet_map.contains_key(&letter) {
	    return None
	}	
	let equivalent = alphabet_map[&letter];
	let current_state_obj = &states[current_state];
	let edges = &current_state_obj.transitions;
	current_state = edges[equivalent];
    }
    println!{"Final state is {}",current_state}
    return Some(states[current_state].accepting)
    
}
