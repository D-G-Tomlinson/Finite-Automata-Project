mod dfa;
mod nfa;
mod regex;
mod int_nfa_reg;

use crate::dfa::DFA;
use crate::nfa::NFA;
use crate::regex::Regex;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use std::convert::From;

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

    /// Regular expression to be evaluated or converted. ':' and ',' are not valid. () is used to alter order of operations, + is any positive number of repitions, * is + but also zero repetitions, ? is zero or one repetitions
    #[arg(short, long)]
    regex: Option<String>,

    /// Location of the DFA file to write the converted input to
    #[arg(short, long)]
    nfa_output: Option<String>,

	/// Flag if a converted regex is desired
	#[arg(long)]
	regex_output: bool,
}

struct Automata {
	dfa: Option<DFA>,
	nfa: Option<NFA>,
	regex: Option<Regex>
}

impl Automata {

	fn new(cli:&Cli) -> Result<Automata,String> {
		let input_type:InputType = match get_input_type(cli) {
			Err(e) => return Err(e),
			Ok(it) => it
		};
		
		let lines:Vec<String>;
		if input_type == InputType::Regex {
			lines=Vec::new();
		} else {
			lines = match read_input_file(cli.input.as_deref().unwrap()) {
				Err(e) => return Err(e),
				Ok(l) => l
			};
		}
		return match input_type {
			InputType::Dfa => Automata::new_dfa(lines),
			InputType::Nfa => Automata::new_nfa(lines),
			InputType::Regex => Automata::new_regex(cli.regex.as_deref().unwrap())
		};
	}
	
	fn new_dfa(lines:Vec<String>) -> Result<Automata,String> {
		let dfa = match DFA::try_from(lines) {
			Err(e) => return Err(e),
			Ok(dfa_in) => Some(dfa_in)
		};
		let nfa = None;
		let regex = None;
		return Ok(Automata{dfa,nfa,regex});
	}

	fn new_nfa(lines:Vec<String>) -> Result<Automata,String> {
		let dfa = None;
		let nfa = match NFA::try_from(lines) {
			Err(e) => return Err(e),
			Ok(nfa_in) => Some(nfa_in)
		};		
		let regex = None;
		return Ok(Automata{dfa,nfa,regex});
	}
	
	fn new_regex(regex_str:&str) -> Result<Automata,String> {
		let dfa = None;
		let nfa = None;
		let regex:Option<Regex> = match Regex::try_from(regex_str.to_string()) {
			Err(e) => return Err(e),
			Ok(reg) => Some(reg)
		};
		return Ok(Automata{dfa,nfa,regex});
	}

	fn run(&mut self, word:&str) -> Rslt {
		if !(self.regex.is_some()||self.nfa.is_some()||self.dfa.is_some()) {
			return Rslt::Err(format!("Automata list is unitialised"));
		}
		if !self.dfa.is_some() {
			if !self.nfa.is_some() {
				self.nfa = Some(NFA::from(self.regex.as_ref().unwrap()));
			}
			self.dfa = Some(DFA::from(self.nfa.as_ref().unwrap()));
		}
		return self.dfa.as_ref().unwrap().run(word);
	}
	
	fn output_dfa(&mut self,address:&str) -> Result<(),String>{
		if !(self.regex.is_some()||self.nfa.is_some()||self.dfa.is_some()) {
			return Err(format!("Automata list is unitialised"));
		}
		if !self.dfa.is_some() {
			if !self.nfa.is_some() {
				self.nfa = Some(NFA::from(self.regex.as_ref().unwrap()));
			}
			self.dfa = Some(DFA::from(self.nfa.as_ref().unwrap()));
		}

		return match print_to_file(self.dfa.as_ref().unwrap().to_string(),address) {
			Ok(()) => {
				println!("DFA written to {}",address);
				Ok(())
			},
			Err(e) => Err(e)
		}
	}
	
	fn output_nfa(&mut self,address:&str) -> Result<(),String> {
		if !(self.regex.is_some()||self.nfa.is_some()||self.dfa.is_some()) {
			return Err(format!("Automata list is unitialised"));
		}
		if !self.nfa.is_some() {
			match self.dfa.is_some() {
				true => self.nfa = Some(NFA::from(self.dfa.as_ref().unwrap())),
				false => self.nfa = Some(NFA::from(self.regex.as_ref().unwrap()))
			}
		}
		return match print_to_file(self.nfa.as_ref().unwrap().to_string(),address) {
			Ok(()) => {
				println!("NFA written to {}",address);
				Ok(())
			},
			Err(e) => Err(e)
		}
	}
	
	fn output_regex(&mut self) -> Result<(),String>{
		if !(self.regex.is_some()||self.nfa.is_some()||self.dfa.is_some()) {
			return Err(format!("Automata list is unitialised"));
		}
		if !self.regex.is_some() {
			if !self.nfa.is_some() {
				self.nfa = Some(NFA::from(self.dfa.as_ref().unwrap()));
			}
			self.regex = Some(Regex::from(self.nfa.as_ref().unwrap()));
		}
		println!("Regex is: {}",self.regex.as_ref().unwrap().to_string());
		return Ok(());
	}
	
}

#[derive(Debug, PartialEq, Eq)]
enum InputType{
    Dfa,
    Nfa,
    Regex
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
    let result=run_automata(&cli);
    println!("{}", match result {
		Rslt::Err(e) => format!("Program failed! The following error was produced: \n{}",e),
		Rslt::Acc => format!("ACCEPT"),
		Rslt::Rej => format!("REJECT"),
		Rslt::Nop => format!("No word provided, program finished without computation, only conversion."),
		Rslt::Notodo => format!("No word or output file provided, nothing to do."),
    });
    return;
}

fn run_automata(cli:&Cli) -> Rslt {

	if !(cli.word.as_deref().is_some() ||cli.dfa_output.as_deref().is_some()||cli.nfa_output.as_deref().is_some()||cli.regex_output) {
		return Rslt::Notodo;
	}
	
	let mut autos:Automata = match Automata::new(cli) {
		Err(e) => return Rslt::Err(e),
		Ok(a) => a
	};

	if cli.regex_output {
		match autos.output_regex() {
			Ok(()) => (),
			Err(e) => return Rslt::Err(e)
		}
	}

	if let Some(address) = cli.nfa_output.as_deref() {
		match autos.output_nfa(address) {
			Ok(()) => (),
			Err(e) => return Rslt::Err(e)
		}
	}

	if let Some(address) = cli.dfa_output.as_deref() {
		match autos.output_dfa(address) {
			Ok(()) => (),
			Err(e) => return Rslt::Err(e)
		}
	}

	if let Some(word) = cli.word.as_deref() {
		return autos.run(word);
	}
	return Rslt::Nop;
}


fn get_input_type(cli:&Cli) -> Result<InputType,String> {
	let is_regex = cli.regex.as_deref().is_some();
	return match &cli.input.as_deref() {
		None => match is_regex {
			true => Ok(InputType::Regex),
			false => Err(format!("No automata or regex provided."))
		},
		Some(address) => match is_regex {
			true => Err(format!("Cannot input both regex and other automata")),
			false => match address.split('.').last().unwrap().to_uppercase().as_str() {
				"DFA" => Ok(InputType::Dfa),
				"NFA" => Ok(InputType::Nfa),
				_ => {
					return Err(format!("File type is unsupported."));
				}				
			}
		}
	}
}

fn read_input_file(address:&str) -> Result<Vec<String>,String> {
    let mut contents= String::new();
    match File::open(address) {
		Ok(mut f) => _ = f.read_to_string(&mut contents),
		Err(_) => {
			return Err(format!("Cannot read file."));
		}
    }
    let lines = contents.lines().map(|s| s.to_string()).collect();
    return Ok(lines);
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


fn get_alphabet(alphabet:&str) -> Result<String,String> {
	let invalid_letters = vec![':',','];
	let mut result:Vec<char> = Vec::new();
	for c in alphabet.chars() {
		if invalid_letters.contains(&c) || Regex::VALID_SYMBOLS.contains(&c){
			return Err(format!("The alphabet cannot contain {}",c));
		} 
		if !result.contains(&c) {
			result.push(c);
		}
	}
	return Ok(result.into_iter().collect());
}

fn get_alphabet_hm(alphabet:&str) -> HashMap<char,Index0> {

	let alphabet = alphabet.chars();
	
	let mut alphabet_hashmap = HashMap::<char,Index0>::new();
	let mut i = Index0(0); //not to be read here, only passed to dfa so ignoring jump is fine
	for c in alphabet {
		if !alphabet_hashmap.contains_key(&c) {
			alphabet_hashmap.insert(c,i);
			i = Index0(i.0 + 1);
		}
	}
	return alphabet_hashmap;
}
#[derive(Clone,Copy,Debug)]
struct Index0(usize); // alphabet indexing with first letter at 0

#[derive(Clone,Copy,Debug)]
struct Index1(usize); // alphabet indexing with first letter at 1, jump at 0

impl From<Index1> for Index0 {
	fn from(i1:Index1) -> Self {
		let Index1(i1) = i1;
		return Self(i1-1);
	}
}
impl From<Index0> for Index1 {
	fn from(i0:Index0) -> Self {
		let Index0(i0) = i0;
		return Self(i0+1);
	}
}

type StateNum = usize;
#[derive(Clone,Debug,Hash,Eq,PartialEq)]
struct Ordered(Vec<StateNum>);
impl Ordered {
	pub fn join(&self,v2:&Self) -> Self {
	let Self(v1) = self;
	let Self(v2) = v2;
	
	if v1.len()==0 {
		return Self(v2.to_vec());
	} else if v2.len()==0 {
		return Self(v1.to_vec());
	}
	
	let mut result:Vec<StateNum> = Vec::new();
	let mut i = 0;
	let mut j = 0;
	if v1[0]<v2[0] {
			result.push(v1[0]);
	} else {
		result.push(v2[0]);
	}
		
	while i < v1.len() || j < v2.len() {
		let consider_v1:bool;
		if i==v1.len() {
			consider_v1 = false;
		} else if j==v2.len() {
			consider_v1 = true;
		} else if v1[i]<=v2[j] {
			consider_v1 = true;
		} else {
			consider_v1 = false;
		}
		if consider_v1 {
			if result.last().unwrap() == &v1[i] {
				i=i+1
			} else {
				result.push(v1[i]);
				i=i+1;
			}
		} else {
			if result.last().unwrap() == &v2[j] {
				j=j+1
			} else {
				result.push(v2[j]);
				j=j+1;
			}
		}
	}
	return Self(result);
}

}
