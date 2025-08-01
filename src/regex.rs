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
			return Err(format!("There is a closing bracket with no matching opening bracket"));
		}
		if !(Self::VALID_SYMBOLS.contains(c)||alphabet.contains(c)) {
			alphabet.push(*c);
		}
		}
		if depth != 0 {
			return Err(format!("There are opening brackets that are not closed"));
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
	    RegexTree::Empty => (),
	    RegexTree::Single(_) => (), //could put these together at the default case, but this nested match is complicated enough that I want each option addressed explicitly
	    RegexTree::KleeneStar(br) => {
		match *br {
		    RegexTree::Empty => {
			input = RegexTree::Empty;
			changed = true;
		    },
		    RegexTree::Single(_) => (),
		    RegexTree::KleeneStar
		    
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

#[derive(Clone,Debug)]
pub enum RegexTree {
    Empty,
    Single(Index0),
    KleeneStar(Box<RegexTree>),
    KleenePlus(Box<RegexTree>),
    QMark(Box<RegexTree>),
    Concat((Box<RegexTree>,Box<RegexTree>)),
    Or((Box<RegexTree>,Box<RegexTree>)),
}
impl RegexTree {
	pub fn to_nfa(&self,a:String) -> Result<NFA,String> {
		return match &self {
			Self::Empty => NFA::get_accept_empty(a),
			Self::Single(i) => NFA::get_accept_single((*i).into(),a),
			Self::KleeneStar(r) => get_kstar(&*r, a),
			Self::KleenePlus(r) => get_concat(&(**r).clone(),&Self::KleeneStar(Box::new((**r).clone())),a),
			Self::QMark(r) => get_or(&Self::Empty,&**r,a),
			Self::Concat((r1,r2)) => get_concat(&**r1, &**r2,a),
			Self::Or((r1, r2)) => get_or(&**r1,&**r2,a)
		};
	}

	fn find_next_close(input:&Vec<InProgress>,start:usize) -> Option<usize> {
		for i in start..input.len() {
			if let InProgress::Close = input[i] {
				return Some(i)
			}
		}
		return None;
	}
	
	fn find_last_open(input:&Vec<InProgress>,start:usize) -> Option<usize> {
		for i in (start-1)..=0 {
			if let InProgress::Open = input[i] {
				return Some(i);
			}
		}
		return None;
	}

	fn process_brackets(input:&mut Vec<InProgress>) {
		let mut start_search = 0;
		while let Some(end) = Self::find_next_close(input, start_search) {
			let start = Self::find_last_open(&input,end).expect("Each closing bracket must have an opening");
			start_search = start+1;

			input.remove(start);
			if start == end -1 {
				input[start] = InProgress::Reg(Self::Empty);
			} else {
				let mut sub_bracket:Vec<InProgress> = Vec::new();
				for _ in start..(end-1) {
					sub_bracket.push(input.remove(start));
				}
				input[start] = InProgress::Reg(Self::from(sub_bracket));
			}
		}
	}

	fn add_unary(input:&mut Vec<InProgress>, i:usize, f:fn(Box<Self>) -> Self) {
		if i == 0 {
			input[0] = InProgress::Reg(Self::Empty);
		} else if let InProgress::Reg(r) = &input[i-1] {
			input[i-1] = InProgress::Reg(f(Box::new(r.clone())));
			input.remove(i);
		} else {
			input[i] = InProgress::Reg(Self::Empty);
		}
	}
	
	fn process_unary(input:&mut Vec<InProgress>) {
		let mut i = 0;
		while i < input.len() {
			match &input[i] {
				InProgress::KStar => Self::add_unary(input, i, |r| Self::KleeneStar(r)),
				InProgress::KPlus => Self::add_unary(input, i, |r| Self::KleenePlus(r)),
				InProgress::QMark => Self::add_unary(input, i, |r| Self::QMark(r)),
				_ => i = i + 1
			}
		}
	}

	fn process_concat(input:&mut Vec<InProgress>) {
		let mut i = 0;
		while i < input.len() - 1 {
			match &input[i] {
				InProgress::Reg(r1) => {
					match &input[i + 1] {
						InProgress::Reg(r2) => {
							input[i] = InProgress::Reg(Self::Concat((Box::new(r1.clone()),Box::new(r2.clone()))));
							input.remove(i + 1);			
						},
						_ => i = i + 1 
					}
				},
				_ => i = i + 1
			}
		}
	}

	fn process_or(input:&mut Vec<InProgress>) {
		let mut i = 0;
		while i < input.len() {
			match &input[i] {
				InProgress::Or => {
					let r1;
					if i == 0 {
						r1 = Self::Empty;
					} else if let InProgress::Reg(temp) = &input[i-1] {
						r1 = (*temp).clone();
						i = i - 1;
						input.remove(i);
					} else {
						r1 = Self::Empty;//this will never be reached, as all other possible InProgress values have been removed
					}					
					let r2;
					if i == input.len() - 1 {
						r2 = Self::Empty;
					} else if let InProgress::Reg(temp) = &input[i + 1] {
						r2 = (*temp).clone();
						input.remove(i + 1);
					} else {
						r2 = Self::Empty;
					}
					let new_val = match r1 {
						Self::Empty => match r2 {
							Self::Empty => Self::Empty,
							r => Self::QMark(Box::new(r))
						},
						r1 => match r2 {
							Self::Empty => Self::QMark(Box::new(r1)),
							r2 => Self::Or((Box::new(r1),Box::new(r2)))
						}
					};
					input[i] = InProgress::Reg(new_val);
				},
				_ => i = i + 1
			}
		}
	}
	
	fn opp_to_string(opp:char, child:&Self,alphabet:&Vec<char>) -> String {//regex is a mix of infix and postfix notation so brackets need to be added where appropriate
		let mut result = String::new();
		//need brackets around ors or concats

		match child {
			Self::Empty => return String::new(),
			Self::Single(_)|Self::KleeneStar(_)|Self::KleenePlus(_)|Self::QMark(_) => result.push_str(&child.to_string(alphabet)),
			Self::Concat(_)|Self::Or(_) => {
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
			Self::Or(_) => {
				format!("({})",r1.to_string(alphabet))
			},
			_ => r1.to_string(alphabet)
		};

		let s2 = match r2 {
			Self::Or(_) => {
				format!("({})",r2.to_string(alphabet))
			},
			_ => r2.to_string(alphabet)
		};
		s1.push_str(&s2);
		return s1;
	}
	
	pub fn to_string(&self, alphabet:&Vec<char>) -> String {
		return match &self {
			Self::Empty => String::new(),
			Self::Single(i) => alphabet[(*i).0].to_string(),
			Self::KleeneStar(r) => Self::opp_to_string('*',&**r,alphabet),
			Self::KleenePlus(r) => Self::opp_to_string('+',&**r,alphabet),
			Self::QMark(r) => Self::opp_to_string('?',&**r,alphabet),
			Self::Concat((r1,r2)) => Self::concat_to_string(&**r1,&**r2,alphabet),
			Self::Or((r1,r2)) => format!("{}|{}",r1.to_string(alphabet),r2.to_string(alphabet)),
		}
	}
}

impl From<Vec<InProgress>> for RegexTree {

	fn from(input:Vec<InProgress>) -> Self {
		if input.len() == 0 {
			return Self::Empty;
		}
		if input.len() == 1 {
			return match &input[0] {
				InProgress::Reg(r) => r.clone(),
				_ => Self::Empty//due to earlier checks, we know brackets match, so do not need to consider them here, and a single unary operator, or | on its own is equivalent to empty
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
		if let InProgress::Reg(r) = &input[0] {
			return r.clone();
		} else {
			return Self::Empty; //can't be reached due to earlier code
		}
	}

}

#[derive(Clone,Debug)]
enum InProgress {
    Reg(RegexTree),
    KStar,
    KPlus,
    QMark,
    Or,
    Open,
    Close
}
impl InProgress {
	fn from_char(c:char, hm:&HashMap<char,Index0>) -> InProgress {
		match c {
			'*' => InProgress::KStar,
			'+' => InProgress::KPlus,
			'?' => InProgress::QMark,
			'|' => InProgress::Or,
			'(' => InProgress::Open,
			')' => InProgress::Close,
			other => InProgress::Reg(RegexTree::Single(hm[&other]))
		}    
	}
	
}
