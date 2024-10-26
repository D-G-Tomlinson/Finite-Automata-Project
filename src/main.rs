use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Location of the file storing the DSA
    #[arg(short,long)]
    dfa: Option<String>,

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
    if let Some(dfa) = cli.dfa.as_deref() {
	address=dfa;
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
    let alphabet = lines[0];
    let mut alphabet_map:HashMap<char,usize> = HashMap::new();

    let mut i:usize = 0;
    for letter in alphabet.chars() {
	alphabet_map.insert(letter,i);
	i = i + 1;
    }

    
    let mut current_state=lines[1].parse::<usize>().unwrap()-1;

    let num_letters = alphabet.len();
    let mut states:Vec<State> = Vec::new();
    
    for state in &lines[2..] {
	let split_state:Vec<&str> = state.split(",").collect();
	let next_states:Vec<usize> = (&split_state[0..num_letters]).into_iter().map(|n| n.parse::<usize>().unwrap()-1).collect();
	let accept: bool = split_state[num_letters].parse().unwrap();
	let new_state =State{
	    transitions: next_states,
	    accepting: accept
	}; 
	states.push(new_state);
    }

    for letter in word.chars() {
	let equivalent = alphabet_map[&letter];
	let current_state_obj = &states[current_state];
	let edges = &current_state_obj.transitions;
	current_state = edges[equivalent];
    }
    println!{"Final state is {}",current_state}
    println!{"{}", match states[current_state].accepting{
	true => "ACCEPT",
	false => "REJECT"
    }};
    return;
}
