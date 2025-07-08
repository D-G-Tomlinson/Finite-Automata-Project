mod dfa;
mod nfa;
mod regex;

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

    /// Location of the DFA file to write the converted input to, for regex or NFA input
    #[arg(short, long)]
    dfa_output: Option<String>,

    /// Regular expression to be evaluated or converted. ':' and ',' are not valid. () is used to alter order of operations, + is any positive number of repitions, * is + but also zero repetitions, ? is zero or one repetitions
    #[arg(short, long)]
    regex: Option<String>,

    /// Location of the DFA file to write the converted input to, for regex or NFA input
    #[arg(short, long)]
    nfa_output: Option<String>,

}
#[derive(Debug, PartialEq, Eq)]
enum InputType{
    DFA,
    NFA,
    REGEX
}

pub enum Rslt {
    Acc,//the word is accepted
    Rej,//the word is rejected
    Nop,//no word is provided "no operation performed"
    Notodo, //nothing to do, no word or output file provided
    Err(String) // some error occured, due to invalid input
}

//Although some input validation is done, it is not comprehensive - this project is an exercise in regular langauge representation, conversion and computation, rather than data validation
fn main() {

    println!("Use the --help option (i.e. cargo run -- --help) to learn about possible options.");
    
    let cli = Cli::parse();

    let mut regex_in:String = String::new();
    let is_regex:bool;
    if let Some(regex) = cli.regex.as_deref() {
	regex_in = String::from(regex);
	is_regex = true;
    } else {
	is_regex = false;
    }

    let input_type:InputType;
    let lines:Vec<String>;
    match get_type_lines(cli.input.as_deref(), is_regex) {
	Err(e) => {
	    println!("{}",e);
	    return;
	},
	Ok((t,l)) => {
	    input_type = t;
	    lines = l;
	}
    }

    let input_word = cli.word.as_deref();
    let output_dfa = cli.dfa_output.as_deref();
    let output_nfa = cli.nfa_output.as_deref();
    
    let result:Rslt = match input_type {
	InputType::DFA => dfa::dfa_option(lines, input_word),
	InputType::NFA => nfa::nfa_option(lines, input_word, output_dfa),
	InputType::REGEX => Rslt::Err(format!("Not implemented yet"))//regex::run_regex(regex_in, output_dfa, output_nfa, input_word)
    };
    
    println!("{}", match result {
	Rslt::Err(e) => format!("Program failed! The following error was produced: \n{}",e),
	Rslt::Acc => format!("ACCEPT"),
	Rslt::Rej => format!("REJECT"),
	Rslt::Nop => format!("No word provided, program finished without computation, only conversion."),
	Rslt::Notodo => format!("No word or output file provided, nothing to do.")
    });
    return;
}

fn get_type_lines(input: Option<&str>, is_regex:bool) -> Result<(InputType, Vec<String>),String> {
    if let Some(address) = input {
	if is_regex {
	    return Err(format!("No input file for regex operations."));
	} else {
	    return read_input_file(address);
	}
    } else if is_regex {
	return Ok((InputType::REGEX, Vec::new()));
    } else {
	return Err(format!("No automata or regex provided."));
    }
}

fn read_input_file(address:&str) -> Result<(InputType, Vec<String>),String> {
    let file_type = address.split('.').last().unwrap().to_uppercase();
    let input_type:InputType;
    match file_type.as_str() {
	"DFA" => input_type = InputType::DFA,
	"NFA" => input_type = InputType::NFA,
	_ => {
	    return Err(format!("File type is unsupported."));
	}
    }    
    let mut contents= String::new();
    match File::open(address) {
		Ok(mut f) => _ = f.read_to_string(&mut contents),
		Err(_) => {
			return Err(format!("Cannot read file."));
		}
    }
    //println!("{}",contents);
    let lines = contents.lines().map(|s| s.to_string()).collect();
    return Ok((input_type,lines));
}

fn print_to_file(val:String,address:&str) -> Result<(),String> {
    if let Ok(mut file_ptr) = File::create(address){
		match write!(file_ptr, "{}", val) {
			Ok(_) => (),
			Err(e) => return Err(e.to_string())
		}
    }
    return Ok(());
}
