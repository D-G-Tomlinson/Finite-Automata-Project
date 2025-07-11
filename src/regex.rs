use std::collections::HashMap;
use crate::Rslt;
use crate::nfa::NFAState;
use crate::nfa::NFA;

struct Regex {
	alphabet:HashMap<char,usize>,
	tree:RegexTree
}

impl Regex {
	pub fn new(alphabet:HashMap<char,usize>, tree:RegexTree) -> Regex {
		Regex{alphabet,tree}
	}
	pub fn from_string(regex_in:String) -> Result<Regex,String> {
		let regex_in:Vec<char>=regex_in.chars().collect();
		let (regex, alphabet): (Vec<char>, String);
		match validate_regex(regex_in) {
			None => return Err(format!("Invalid regex")),
			Some((a,b)) => (regex,alphabet) = (a,b)
		}
		let collected_alphabet:Vec<char> = alphabet.chars().collect();
		let mut alphabet_hashmap:HashMap<char,usize> = HashMap::new();
		let mut i = 0;
		for c in collected_alphabet {
			alphabet_hashmap.insert(c,i);
			i = i + 1;
		}
		
		let regex:Vec<InProgress> = regex.iter().map(|c| char_to_in_progress(*c,&alphabet_hashmap)).collect();
		
		let regex = in_progress_vec_to_regex(regex);
		return Ok(Regex::new(alphabet_hashmap, regex));
	}

	pub fn to_nfa(&self) -> NFA{
		let nfa_without_alphabet = self.tree.to_nfa(self.alphabet.len()+1);
		return NFA::new(nfa_without_alphabet.states,nfa_without_alphabet.starting,self.alphabet.clone());
	}
	
	pub fn run(&self,nfa_address:Option<&str>,dfa_address:Option<&str>,word:Option<&str>) -> Rslt {
		let result_nfa = self.to_nfa();
		if nfa_address.is_some() {	
			match crate::print_to_file(result_nfa.to_string(),nfa_address.unwrap()) {
				Ok(()) => (),
				Err(e) => return Rslt::Err(e)
			}
		}
		
		if word.is_some() || dfa_address.is_some() {
			return result_nfa.run(word,dfa_address);
		}
		
		return Rslt::Nop;
		
	}
	
}

pub fn regex_option(regex_in:String, output_dfa:Option<&str>, output_nfa_in:Option<&str>, input_word:Option<&str>) -> Rslt {
    // the NFA to DFA step will validate this again, but it's useful to validate this here - if no valid output or word is provided the program doesn't need to waste time converting the regex to NFA
	//once regex is refactored I'll change all of them so these steps are done in main
    if let Some(ref in_output) = output_dfa {
		let file_type = in_output.split('.').last().unwrap().to_uppercase();
		if file_type != "DFA" {
			return Rslt::Err(format!("Can only DFA output write to .dfa files"));
		}
    }
	

    if let Some(in_output) = output_nfa_in {
		let file_type = in_output.split('.').last().unwrap().to_uppercase();
		if file_type != "NFA" {
			return Rslt::Err(format!("Can only NFA output write to .nfa files"));
		}
    }
	
    if !(output_dfa.is_some()||input_word.is_some()||output_nfa_in.is_some()) {
		return Rslt::Notodo;
    }

	let regex:Regex = match Regex::from_string(regex_in) {
		Ok(reg) => reg,
		Err(e) => return Rslt::Err(e)
	};
    
	return regex.run(output_nfa_in,output_dfa,input_word);
	
}

fn validate_regex(regex:Vec<char>) -> Option<(Vec<char>,String)> {
    //valid characters are (,),|,+,?,*, and all lowercase ascii letters other than ':' or ','
    let valid_symbols = vec!['(',')','|','+','?','*'];
    let invalid_letters = vec![':',','];
    
    let mut alphabet:Vec<char> = Vec::new();
    let mut depth=0;
    for c in &regex {
	if invalid_letters.contains(c) {
	    return None;
	}
	if *c == '(' {
	    depth += 1;
	} else if *c == ')' {
	    depth -= 1;
	}
	if depth == -1 {
	    return None;
	}
	/*
	if !(*c >= 'a' && *c <= 'z'){
	    return None;
	}
	*/
	if !(valid_symbols.contains(c)||alphabet.contains(c)) {
	    alphabet.push(*c);
	}
    }
    if depth != 0 {
	return None;
    }
    return Some((regex, alphabet.iter().cloned().collect()));
}

fn char_to_in_progress(c:char, hm:&HashMap<char,usize>) -> InProgress {
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

fn in_progress_vec_to_regex(input:Vec<InProgress>) -> RegexTree {
    if input.len() == 0 {
	return RegexTree::Empty;
    }
    if input.len() == 1 {
	return match &input[0] {
	    InProgress::Reg(r) =>  r.clone(),
	    _ => RegexTree::Empty//due to earlier checks, we know brackets match, so do not need to consider them here, and a single unary operator, or | on its own is equivalent to empty
	};
    }

    let mut input = input.clone();

    //brackets have priority
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
				input[j] = InProgress::Reg(in_progress_vec_to_regex(sub_bracket));
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

    //unary operators are next
    i = 0;
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
    
    //now, all we're left with is InProgress::Regs and InProgress::Ors
    i = 0;
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

    //now just to deal with the Ors
    i = 0;
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

    if input.len() != 1 {
	panic!("Regex parsing process finished with incorrect number of results");
    } else if let InProgress::Reg(r) = &input[0] {
	return (*r).clone();
    } else {
	panic!("Regex parsing process finished without a regex");
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
struct NFAForRegex {
    states:Vec<NFAState>,
    starting:usize
}

fn get_accept_e(alphabet_len:usize) -> NFAForRegex {
    let state = NFAState {
	transitions:vec![Vec::<usize>::new();alphabet_len],
	accepting:true
    };
    NFAForRegex {
	states: vec![state],
	starting: 0
    }
}

fn get_accept_single(i:usize,alphabet_len:usize) -> NFAForRegex {
    let mut transitions = vec![Vec::<usize>::new();alphabet_len];
    transitions[i] = vec![1];
    let start = NFAState {
	transitions,
	accepting:false
    };
    let end = NFAState {
	transitions:vec![Vec::<usize>::new();alphabet_len],
	accepting:true
    };
    NFAForRegex {
	states:vec![start,end],
	starting: 0
    }
}

fn get_kstar(r:&RegexTree, alphabet_len:usize ) -> NFAForRegex {
    let mut sub =  r.to_nfa(alphabet_len);
    let start = sub.starting;
    let len = sub.states.len();
    for s in &mut sub.states {
	if s.accepting {
	    s.transitions[0].push(len);
	    s.accepting = false;
	}
    }
    let mut new_transitions = vec![Vec::new();alphabet_len];
    new_transitions[0].push(start);
    sub.states.push(NFAState{
	transitions:new_transitions,
	accepting:true
    });
    sub.starting = len;
    return sub;
}

fn get_concat(r1:&RegexTree, r2:&RegexTree, alphabet_len:usize) -> NFAForRegex {
    let mut r1 = r1.to_nfa(alphabet_len);
    let mut r2 = r2.to_nfa(alphabet_len);

    let num_states = r1.states.len();
    let second_start = r2.starting + num_states;

    for state in &mut r1.states {
	if state.accepting {
	    state.transitions[0].push(second_start);
	    state.accepting = false;
	}
    }

    for state in &mut r2.states {
	for t in &mut state.transitions {
	    for i in t {
		*i = *i + num_states;
	    }
	}
    }
    r1.states.append(&mut r2.states);
    return r1;
}

fn get_or(r1:&RegexTree, r2:&RegexTree, alphabet_len:usize) -> NFAForRegex {
    let mut r1 = r1.to_nfa(alphabet_len);
    let mut r2 = r2.to_nfa(alphabet_len);

    let num_states = r1.states.len();
    let second_start = r2.starting + num_states;

    for state in &mut r2.states {
	for t in &mut state.transitions {
	    for i in t {
		*i = *i + num_states;
	    }
	}
    }
    r1.states.append(&mut r2.states);
    let mut new_transitions = vec![Vec::<usize>::new();alphabet_len];
    new_transitions[0].push(r1.starting);
    new_transitions[0].push(second_start);

    r1.starting = r1.states.len();
    r1.states.push(NFAState {
	transitions:new_transitions,
	accepting:false	
    });
    return r1;
}

fn nfa_for_regex_to_nfa(regex:NFAForRegex,alphabet:String) -> Vec<String> {
    let mut result:Vec<String> = Vec::new();
    result.push(alphabet.clone());    
    let alphabet:Vec<char> = alphabet.chars().collect();

    result.push((regex.starting + 1).to_string());

    for state in regex.states {
	println!("State is {state:?}");
	let mut line = String::new();
	//
	for i in &state.transitions[0] {
	    line.push_str(&format!(":{},",i+1));
	}
	
	for t in 0..alphabet.len() {
	    for i in &state.transitions[t+1] {
		line.push_str(&format!("{}:{},",alphabet[t],i+1));
	    }
	}
	line.push_str(&state.accepting.to_string());
	result.push(line);
    }
    
    return result;
}

#[derive(Clone,Debug)]
enum RegexTree {
    Empty,
    Single(usize),
    KleeneStar(Box<RegexTree>),
    KleenePlus(Box<RegexTree>),
    QMark(Box<RegexTree>),
    Concat((Box<RegexTree>,Box<RegexTree>)),
    Or((Box<RegexTree>,Box<RegexTree>)),
}
impl RegexTree {
	fn to_nfa(&self,a:usize) -> NFAForRegex {
		match &self {
			RegexTree::Empty => get_accept_e(a),
			RegexTree::Single(i) => get_accept_single(*i,a),
			RegexTree::KleeneStar(r) => get_kstar(&*r, a),
			RegexTree::KleenePlus(r) => get_concat(&(**r).clone(),&RegexTree::KleeneStar(Box::new((**r).clone())),a),
			RegexTree::QMark(r) => get_or(&RegexTree::Empty,&**r,a),
			RegexTree::Concat((r1,r2)) => get_concat(&**r1, &**r2,a),
			RegexTree::Or((r1, r2)) => get_or(&**r1,&**r2,a)
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
