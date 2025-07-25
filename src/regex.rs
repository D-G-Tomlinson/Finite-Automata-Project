use std::fmt;
use std::collections::HashMap;
use crate::nfa::NFA;

use std::convert::From;
use std::convert::TryFrom;

#[derive(Clone,Debug)]
pub struct Regex {
	alphabet:String,
	tree:Option<RegexTree>
}

impl Regex {
	pub fn new(alphabet:String, tree:Option<RegexTree>) -> Regex {
		Regex{alphabet,tree}
	}
	pub fn to_nfa(&self) -> NFA{
		return match &self.tree {
			None => NFA::get_never_accept(self.alphabet.clone()),
			Some(tree) => tree.to_nfa(self.alphabet.clone()).expect("This only fails if two generated alphabets are different, which indicates a programming error, not a user error")
		};
	}
	
	fn validate_regex(regex:&Vec<char>) -> Result<String,String> {
		//valid characters are (,),|,+,?,*, and all lowercase ascii letters other than ':' or ','
		let valid_symbols = vec!['(',')','|','+','?','*'];
		
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
		if !(valid_symbols.contains(c)||alphabet.contains(c)) {
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
		let alphabet: String =	match Regex::validate_regex(&regex) {
			Err(e) => return Err(format!("Invalid regex. {}",e)),
			Ok(a) => a
		};
		let alphabet = match crate::get_alphabet(&alphabet) {
			Err(e) => return Err(e),
			Ok(ab) => ab
		};
		let alphabet_hashmap = crate::get_alphabet_hm(&alphabet);
		let regex:Vec<InProgress> = regex.iter().map(|c| InProgress::from_char(*c,&alphabet_hashmap)).collect();
		
		let regex = RegexTree::from_in_progress(regex);
		return Ok(Regex::new(alphabet, Some(regex)));
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
fn simplify_regex(mut regex: Regex) -> Regex {//this step isn't strictly necessary, but it should simplify the resultant Automata, which is useful with the 64 state limit in NFA.Ccan be done at a later stage of the project
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
    Single(usize),
    KleeneStar(Box<RegexTree>),
    KleenePlus(Box<RegexTree>),
    QMark(Box<RegexTree>),
    Concat((Box<RegexTree>,Box<RegexTree>)),
    Or((Box<RegexTree>,Box<RegexTree>)),
}
impl RegexTree {
	fn to_nfa(&self,a:String) -> Result<NFA,String> {
		return match &self {
			RegexTree::Empty => NFA::get_accept_empty(a),
			RegexTree::Single(i) => NFA::get_accept_single(*i,a),
			RegexTree::KleeneStar(r) => get_kstar(&*r, a),
			RegexTree::KleenePlus(r) => get_concat(&(**r).clone(),&RegexTree::KleeneStar(Box::new((**r).clone())),a),
			RegexTree::QMark(r) => get_or(&RegexTree::Empty,&**r,a),
			RegexTree::Concat((r1,r2)) => get_concat(&**r1, &**r2,a),
			RegexTree::Or((r1, r2)) => get_or(&**r1,&**r2,a)
		};
	}

	fn process_brackets(input:&mut Vec<InProgress>) {
		let mut i =0;
		while i < input.len() {
			match &input[i] {
				InProgress::Close => { //as this is the first closing bracket, there are no sub-brackets in it
					let mut j = i - 1; //find start of the bracket
					loop {
						match &input[j] {
							InProgress::Open => {
								input.remove(j);
								if j == i - 1 {
									input[j] = InProgress::Reg(RegexTree::Empty);
								} else {
									let mut sub_bracket:Vec<InProgress> = Vec::new();
									
									for _ in j..(i-1) {
										sub_bracket.push(input[j].clone());
										input.remove(j);
									}
									input[j] = InProgress::Reg(RegexTree::from_in_progress(sub_bracket));
								}
								i = j + 1;
								j = 0;
							},
							_ => {
								if j == 0 { // can't test this in a while loop, as j is usize which cannot be negative
									break;
								} else {
									j = j - 1;
								}
							}
						}
						
					}
				},
				_ => i = i + 1
			}
		}
		
		
	}

	fn process_unary(input:&mut Vec<InProgress>) {
		let mut i = 0;
		while i < input.len() {
			match &input[i] {
				InProgress::KStar => {
					if i == 0 {
						input[0] = InProgress::Reg(RegexTree::Empty);
						i = i + 1
					} else if let InProgress::Reg(r) = &input[i-1] {
						input[i-1] = InProgress::Reg(RegexTree::KleeneStar(Box::new(r.clone())));
						input.remove(i);
					} else {
						input[i] = InProgress::Reg(RegexTree::Empty);
					}
				},
				InProgress::KPlus => {
					if i == 0 {
						input[0] = InProgress::Reg(RegexTree::Empty);
					} else if let InProgress::Reg(r) = &input[i-1] {
						input[i-1] = InProgress::Reg(RegexTree::KleenePlus(Box::new(r.clone())));
						input.remove(i);
					} else {
						input[i] = InProgress::Reg(RegexTree::Empty);
					}
				},
				InProgress::QMark => {
					if i == 0 {
						input[0] = InProgress::Reg(RegexTree::Empty);
					} else if let InProgress::Reg(r) = &input[i-1] {
						input[i-1] = InProgress::Reg(RegexTree::QMark(Box::new(r.clone())));
						input.remove(i);
					} else {
						input[i] = InProgress::Reg(RegexTree::Empty);
					}
				},
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
							input[i] = InProgress::Reg(RegexTree::Concat((Box::new(r1.clone()),Box::new(r2.clone()))));
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
						r1 = RegexTree::Empty;
					} else if let InProgress::Reg(temp) = &input[i-1] {
						r1 = (*temp).clone();
						i = i - 1;
						input.remove(i);
					} else {
						r1 = RegexTree::Empty;//this will never be reached, as all other possible InProgress values have been removed
					}					
					let r2;
					if i == input.len() - 1 {
						r2 = RegexTree::Empty;
					} else if let InProgress::Reg(temp) = &input[i + 1] {
						r2 = (*temp).clone();
						input.remove(i + 1);
					} else {
						r2 = RegexTree::Empty;
					}
					let new_val = match r1 {
						RegexTree::Empty => match r2 {
							RegexTree::Empty => RegexTree::Empty,
							r => RegexTree::QMark(Box::new(r))
						},
						r1 => match r2 {
							RegexTree::Empty => RegexTree::QMark(Box::new(r1)),
							r2 => RegexTree::Or((Box::new(r1),Box::new(r2)))
						}
					};
					input[i] = InProgress::Reg(new_val);
				},
				_ => i = i + 1
			}
		}
	}
	
	fn from_in_progress(input:Vec<InProgress>) -> RegexTree {
		if input.len() == 0 {
			return RegexTree::Empty;
		}
		if input.len() == 1 {
			return match &input[0] {
				InProgress::Reg(r) => r.clone(),
				_ => RegexTree::Empty//due to earlier checks, we know brackets match, so do not need to consider them here, and a single unary operator, or | on its own is equivalent to empty
			};
		}
		let mut input = input.clone();
		//brackets have priority
		RegexTree::process_brackets(&mut input);
		//unary operators are next
		RegexTree::process_unary(&mut input);
		//now, all we're left with is InProgress::Regs and InProgress::Ors
		RegexTree::process_concat(&mut input);
		//now just to deal with the Ors
		RegexTree::process_or(&mut input);
		if let InProgress::Reg(r) = &input[0] {
			return r.clone();
		} else {
			return RegexTree::Empty; //can't be reached due to earlier code
		}
	}

	fn opp_to_string(opp:char, child:&RegexTree,alphabet:&Vec<char>) -> String {//regex is a mix of infix and postfix notation so brackets need to be added where appropriate
		let mut result = String::new();
		//need brackets around ors or concats

		match child {
			RegexTree::Empty => return String::new(),
			RegexTree::Single(_)|RegexTree::KleeneStar(_)|RegexTree::KleenePlus(_)|RegexTree::QMark(_) => result.push_str(&child.to_string(alphabet)),
			RegexTree::Concat(_)|RegexTree::Or(_) => {
				result.push('(');
				result.push_str(&child.to_string(alphabet));
				result.push(')');
			},
		}
		result.push(opp);
		return result;
	}

	fn concat_to_string(r1:&RegexTree,r2:&RegexTree,alphabet:&Vec<char>) -> String{
		let mut s1 = match r1 {
			RegexTree::Or(_) => {
				format!("({})",r1.to_string(alphabet))
			},
			_ => r1.to_string(alphabet)
		};

		let s2 = match r2 {
			RegexTree::Or(_) => {
				format!("({})",r2.to_string(alphabet))
			},
			_ => r2.to_string(alphabet)
		};
		s1.push_str(&s2);
		return s1;
	}
	
	pub fn to_string(&self, alphabet:&Vec<char>) -> String {
		return match &self {
			RegexTree::Empty => String::new(),
			RegexTree::Single(i) => alphabet[*i].to_string(),
			RegexTree::KleeneStar(r) => RegexTree::opp_to_string('*',&**r,alphabet),
			RegexTree::KleenePlus(r) => RegexTree::opp_to_string('+',&**r,alphabet),
			RegexTree::QMark(r) => RegexTree::opp_to_string('?',&**r,alphabet),
			RegexTree::Concat((r1,r2)) => RegexTree::concat_to_string(&**r1,&**r2,alphabet),
			RegexTree::Or((r1,r2)) => format!("{}|{}",r1.to_string(alphabet),r2.to_string(alphabet)),
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
	fn from_char(c:char, hm:&HashMap<char,usize>) -> InProgress {
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
