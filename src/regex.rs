use std::fmt;
use std::collections::HashMap;
use crate::nfa::NFA;

use std::convert::From;
use std::convert::TryFrom;

use crate::Index0;

#[derive(Clone,Debug)]
pub struct Regex {
	pub alphabet:String,
	pub tree:Option<RegexTree>
}

impl Regex {
	//valid characters are (,),|,+,?,*, and all lowercase ascii letters other than ':' or ','	
	pub const VALID_SYMBOLS:[char;6] = ['(',')','|','+','?','*'];

	pub fn new(alphabet:String, tree:Option<RegexTree>) -> Self {
		Self{alphabet,tree}
	}
	fn validate_regex(regex:&Vec<char>) -> Result<String,String> {
		let mut alphabet:Vec<char> = Vec::new();
		let mut depth=0;
		for c in regex {
			if *c == '(' {
				depth += 1;
			} else if *c == ')' {
				depth -= 1;
			}
		if depth == -1 {
			return Err("There is a closing bracket with no matching opening bracket".to_string());
		}
		if !(Self::VALID_SYMBOLS.contains(c)||alphabet.contains(c)) {
			alphabet.push(*c);
		}
		}
		if depth != 0 {
			return Err("There are opening brackets that are not closed".to_string());
		}
		return Ok(alphabet.iter().cloned().collect());
	}

}

impl TryFrom<String> for Regex {
	type Error = String;
	
	fn try_from(regex_in:String) -> Result<Self,Self::Error> {
		let regex:Vec<char>=regex_in.chars().collect();
		let alphabet: String =	match Self::validate_regex(&regex) {
			Err(e) => return Err(format!("Invalid regex. {}",e)),
			Ok(a) => a
		};
		let alphabet = match crate::get_alphabet(&alphabet) {
			Err(e) => return Err(e),
			Ok(ab) => ab
		};
		let alphabet_hashmap = crate::get_alphabet_hm(&alphabet);
		let regex:Vec<InProgress> = regex
			.iter()
			.map(|c| InProgress::from_char(*c,&alphabet_hashmap)).collect();
		
		let regex = RegexTree::from(regex);
		return Ok(Self::new(alphabet, Some(regex)));
	}
}

impl From<&NFA> for Regex {
	fn from(nfa:&NFA) -> Self {
		return crate::int_nfa_reg::nfa_to_regex(nfa);
	}
}

impl fmt::Display for Regex {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let alphabet = &self.alphabet;
		let output = match &self.tree {
			None => String::new(),
			Some(tree) => tree.to_string(&alphabet.chars().collect())
		};
		write!(f,"{}",output)
	}
}

/*
fn simplify_regex(mut regex: Regex) -> Regex {//this step isn't strictly necessary, but it should simplify the resultant NFA
    let mut changed = true;
    while changed {
	changed = false;
	match regex {
	    Empty => (),
	    Single(_) => (), //could put these together at the default case, but this nested match is complicated enough that I want each option addressed explicitly
	    KleeneStar(br) => {
		match *br {
		    Empty => {
			input = Empty;
			changed = true;
		    },
		    Single(_) => (),
		    KleeneStar
		    
		} 
	    }
	}
    }
    return regex
}
*/
fn get_kstar(r:&RegexTree, alphabet:String) -> Result<NFA,String> {
    let mut sub = match r.to_nfa(alphabet) {
		Ok(r) => r,
		Err(e) => return Err(e)
	};
	sub.make_kstar();
    return Ok(sub);
}

fn get_concat(r1:&RegexTree, r2:&RegexTree, alphabet:String) -> Result<NFA,String> {
    let mut r1 = match r1.to_nfa(alphabet.clone()) {
		Ok(r) => r,
		Err(e) => return Err(e)
	};
    let mut r2 = match r2.to_nfa(alphabet) {
		Ok(r) => r,
		Err(e) => return Err(e)
	};
	return NFA::concat(&mut r1,&mut r2);
}

fn get_or(r1:&RegexTree, r2:&RegexTree, alphabet:String) -> Result<NFA,String> {
	    let mut r1 = match r1.to_nfa(alphabet.clone()) {
		Ok(r) => r,
		Err(e) => return Err(e)
	};
    let mut r2 = match r2.to_nfa(alphabet.clone()) {
		Ok(r) => r,
		Err(e) => return Err(e)
	};
	return NFA::or(&mut r1,&mut r2);
}

type BRT = Box<RegexTree>;

use crate::regex::RegexTree::{Empty, Single,KleeneStar,KleenePlus,Optional,Concat,Or};
#[derive(Clone,Debug)]
pub enum RegexTree {
    Empty,
    Single(Index0),
    KleeneStar(BRT),
    KleenePlus(BRT),
    Optional(BRT),
    Concat((BRT,BRT)),
    Or((BRT,BRT)),
}
impl RegexTree {
	pub fn to_nfa(&self,a:String) -> Result<NFA,String> {
		return match &self {
			Empty => NFA::get_accept_empty(a),
			Single(i) => NFA::get_accept_single((*i).into(),a),
			KleeneStar(r) => get_kstar(&*r, a),
			KleenePlus(r) => get_concat(&(**r).clone(),&KleeneStar(Box::new((**r).clone())),a),
			Optional(r) => get_or(&Empty, &**r, a),
			Concat((r1,r2)) => get_concat(&**r1, &**r2,a),
			Or((r1, r2)) => get_or(&**r1,&**r2,a)
		};
	}

	fn process_brackets(input:&mut Vec<InProgress>) {
		for start in (0..input.len()).rev() {
			if let Open = input[start] {
				let mut sub_bracket:Vec<InProgress> = Vec::new();
				loop {
					match input.remove(start+1) {
						Close => break,
						r => sub_bracket.push(r)
					}
				}
				input[start] = Reg(Self::from(sub_bracket));
			}
		}
	}

	fn add_unary(input:&mut Vec<InProgress>, i:usize, f:fn(Box<Self>) -> Self) {
		if i == 0 {
			input[0] = Reg(Empty);
		} else if let Reg(r) = &input[i-1] {
			input[i-1] = Reg(f(Box::new(r.clone())));
			input.remove(i);
		} else {
			input[i] = Reg(Empty);
		}
	}
	
	fn process_unary(input:&mut Vec<InProgress>) {
		for i in (0..input.len()).rev() {
			match &input[i] {
				Asterisk => Self::add_unary(input, i, |r| KleeneStar(r)),
				Plus => Self::add_unary(input, i, |r| KleenePlus(r)),
				QMark => Self::add_unary(input, i, |r| Optional(r)),
				_ => ()
			}
		}
	}

	fn process_concat(input:&mut Vec<InProgress>) {
		for i in (1..input.len()).rev() {
			if let Reg(r2) = &input[i] {
				if let Reg(r1) = &input[i-1] {
					input[i-1] = Reg(Concat((Box::new(r1.clone()),Box::new(r2.clone()))));
					input.remove(i);
				}
			}
		}
	}

	fn process_or(input:&mut Vec<InProgress>) {
		let mut i = 0;
		while i < input.len() {
			if let Pipe = input[i] {
				let r1;
				if i == 0 {
					r1 = Empty;
				} else if let Reg(temp) = &input[i-1] {
					r1 = (*temp).clone();
						input.remove(i);
					i = i - 1; // so i is still pointing to the Or.
				} else {
					r1 = Empty;//this will never be reached, as all other possible InProgress values have been removed
					}					
				let r2;
				if i == input.len() - 1 {
					r2 = Empty;
				} else if let Reg(temp) = &input[i + 1] {
					r2 = (*temp).clone();
					input.remove(i + 1);
				} else {
						r2 = Empty;// this could be another Or though
				}
				let new_val = match r1 {
					Empty => match r2 {
						Empty => Empty,
						r => Optional(Box::new(r))
					},
					r1 => match r2 {
						Empty => Optional(Box::new(r1)),
							r2 => Or((Box::new(r1),Box::new(r2)))
					}
				};
				input[i] = Reg(new_val);
			}
			i = i + 1;
		}
	}
	
	fn opp_to_string(opp:char, child:&Self,alphabet:&Vec<char>) -> String {//regex is a mix of infix and postfix notation so brackets need to be added where appropriate
		let mut result = String::new();
		//need brackets around ors or concats

		match child {
			Empty => return String::new(),
			Single(_)|KleeneStar(_)|KleenePlus(_)|Optional(_) => result.push_str(&child.to_string(alphabet)),
			Concat(_)|Or(_) => {
				result.push('(');
				result.push_str(&child.to_string(alphabet));
				result.push(')');
			},
		}
		result.push(opp);
		return result;
	}

	fn concat_to_string(r1:&Self,r2:&Self,alphabet:&Vec<char>) -> String{
		let mut s1 = match r1 {
			Or(_) => {
				format!("({})",r1.to_string(alphabet))
			},
			_ => r1.to_string(alphabet)
		};

		let s2 = match r2 {
			Or(_) => {
				format!("({})",r2.to_string(alphabet))
			},
			_ => r2.to_string(alphabet)
		};
		s1.push_str(&s2);
		return s1;
	}
	
	pub fn to_string(&self, alphabet:&Vec<char>) -> String {
		return match &self {
			Empty => String::new(),
			Single(i) => alphabet[(*i).0].to_string(),
			KleeneStar(r) => Self::opp_to_string('*',&**r,alphabet),
			KleenePlus(r) => Self::opp_to_string('+',&**r,alphabet),
			Optional(r) => Self::opp_to_string('?', &**r, alphabet),
			Concat((r1,r2)) => Self::concat_to_string(&**r1,&**r2,alphabet),
			Or((r1,r2)) => format!("{}|{}",r1.to_string(alphabet),r2.to_string(alphabet)),
		}
	}
}

impl From<Vec<InProgress>> for RegexTree {

	fn from(input:Vec<InProgress>) -> Self {
		if input.len() == 0 {
			return Empty;
		}
		if input.len() == 1 {
			return match &input[0] {
				Reg(r) => r.clone(),
				_ => Empty//due to earlier checks, we know brackets match, so do not need to consider them here, and a single unary operator, or | on its own is equivalent to empty
			};
		}
		let mut input = input.clone();
		//brackets have priority
		Self::process_brackets(&mut input);
		//unary operators are next
		Self::process_unary(&mut input);
		//now, all we're left with is InProgress::Regs and InProgress::Ors
		Self::process_concat(&mut input);
		//now just to deal with the Ors
		Self::process_or(&mut input);
		return if let Reg(r) = &input[0] {
			r.clone()
		} else {
			Empty //can't be reached due to earlier code
		}
	}

}

use crate::regex::InProgress::{Reg,Asterisk,Plus,QMark,Pipe,Open,Close};
#[derive(Clone,Debug)]
enum InProgress {
    Reg(RegexTree),
	Asterisk,
	Plus,
    QMark,
	Pipe,
    Open,
    Close
}
impl InProgress {
	fn from_char(c:char, hm:&HashMap<char,Index0>) -> InProgress {
		match c {
			'*' => Asterisk,
			'+' => Plus,
			'?' => QMark,
			'|' => Pipe,
			'(' => Open,
			')' => Close,
			other => Reg(Single(hm[&other]))
		}    
	}
	
}
