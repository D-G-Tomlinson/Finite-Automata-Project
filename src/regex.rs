use std::collections::HashMap;
use crate::Rslt;

pub fn run_regex(regex_in:String, output_dfa:Option<&str>, output_nfa_in:Option<&str>, input_word:Option<&str>) -> Rslt {
    let is_word:bool;
    //println!("{:?}",word);
    if let Some(_) = input_word {
	is_word = true;
    } else {
	is_word = false;
    }


    // the NFA to DFA step will validate this again, but it's useful to validate this here - if no valid output or word is provided the program doesn't need to waste time converting the regex to NFA
    let is_output_dfa:bool;

    if let Some(ref in_output) = output_dfa {
	let file_type = in_output.split('.').last().unwrap().to_uppercase();
	if file_type != "DFA" {
	    return Rslt::Err(format!("Can only DFA output write to .dfa files"));
	}
	is_output_dfa = true;
    } else {
	is_output_dfa = false;
    }

    let nfa_output_address;
    let is_output_nfa:bool;

    if let Some(in_output) = output_nfa_in {
	nfa_output_address = in_output;
	let file_type = nfa_output_address.split('.').last().unwrap().to_uppercase();
	if file_type != "NFA" {
	    return Rslt::Err(format!("Can only NFA output write to .nfa files"));
	}
	is_output_nfa = true;
    } else {
	nfa_output_address = format!("");
	is_output_nfa = false;
    }

    if !(is_output_dfa||is_word||is_output_nfa) {
	return Rslt::Notodo;
    }

    let regex_in:Vec<char>=regex_in.chars().collect();

    let (regex, alphabet): (Vec<char>, String);
    match validate_regex(regex_in) {
	None => return Rslt::Err(format!("Invalid regex")),
	Some((a,b)) => (regex,alphabet) = (a,b)
    }
    let collected_alphabet:Vec<char> = alphabet.chars().collect();
    let mut alphabet_hashmap:HashMap<char,usize> = HashMap::new();
    let mut i = 1;
    for c in collected_alphabet {
	alphabet_hashmap.insert(c,i);
	i = i + 1;
    }
    
    let regex:Vec<InProgress> = regex.iter().map(|c| char_to_in_progress(*c,&alphabet_hashmap)).collect();

    let regex = in_progress_vec_to_regex(regex);
    //println!("{:?}",regex);    
    let regex = regex_to_nfa(regex,alphabet.len()+1);

    let result_nfa:Vec<String> = nfa_for_regex_to_nfa(regex,alphabet);
    
    if is_output_nfa {
	
	match crate::print_lines_to_file(result_nfa,nfa_output_address) {
	    Ok(()) => (),
	    Err(e) => return Rslt::Err(e)
	}
    }
    
    if is_word || is_output_dfa {
	return run_nfa(regex_nfa,input_word,output_dfa);
    }
    
    return Rslt::Nop;
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
	other => InProgress::Reg(Regex::Single(hm[&other]))
    }    
}

fn in_progress_vec_to_regex(input:Vec<InProgress>) -> Regex {
    if input.len() == 0 {
	return Regex::Empty;
    }
    if input.len() == 1 {
	return match &input[0] {
	    InProgress::Reg(r) =>  r.clone(),
	    _ => Regex::Empty//due to earlier checks, we know brackets match, so do not need to consider them here, and a single unary operator, or | on its own is equivalent to empty
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
				input[j] = InProgress::Reg(Regex::Empty);
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
		    input[0] = InProgress::Reg(Regex::Empty);
		    i = i + 1
		} else if let InProgress::Reg(r) = &input[i-1] {
		    input[i-1] = InProgress::Reg(Regex::KleeneStar(Box::new(r.clone())));
		    input.remove(i);
		} else {
		    input[i] = InProgress::Reg(Regex::Empty);
		}
	    },
	    InProgress::KPlus => {
		if i == 0 {
		    input[0] = InProgress::Reg(Regex::Empty);
		} else if let InProgress::Reg(r) = &input[i-1] {
		    input[i-1] = InProgress::Reg(Regex::KleenePlus(Box::new(r.clone())));
		    input.remove(i);
		} else {
		    input[i] = InProgress::Reg(Regex::Empty);
		}
	    },
	    InProgress::QMark => {
		if i == 0 {
		    input[0] = InProgress::Reg(Regex::Empty);
		} else if let InProgress::Reg(r) = &input[i-1] {
		    input[i-1] = InProgress::Reg(Regex::QMark(Box::new(r.clone())));
		    input.remove(i);
		} else {
		    input[i] = InProgress::Reg(Regex::Empty);
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
			input[i] = InProgress::Reg(Regex::Concat((Box::new(r1.clone()),Box::new(r2.clone()))));
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
		    r1 = Regex::Empty;
		} else if let InProgress::Reg(temp) = &input[i-1] {
		    r1 = (*temp).clone();
		    i = i - 1;
		    input.remove(i);
		} else {
		    r1 = Regex::Empty;//this will never be reached, as all other possible InProgress values have been removed
		}

		let r2;
		if i == input.len() - 1 {
		    r2 = Regex::Empty;
		} else if let InProgress::Reg(temp) = &input[i + 1] {
		    r2 = (*temp).clone();
		    input.remove(i + 1);
		} else {
		    r2 = Regex::Empty;
		}

		let new_val = match r1 {
		    Regex::Empty => match r2 {
			Regex::Empty => Regex::Empty,
			r => Regex::QMark(Box::new(r))
		    },
		    r1 => match r2 {
			Regex::Empty => Regex::QMark(Box::new(r1)),
			r2 => Regex::Or((Box::new(r1),Box::new(r2)))
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
	    Regex::Empty => (),
	    Regex::Single(_) => (), //could put these together at the default case, but this nested match is complicated enough that I want each option addressed explicitly
	    Regex::KleeneStar(br) => {
		match *br {
		    Regex::Empty => {
			input = Regex::Empty;
			changed = true;
		    },
		    Regex::Single(_) => (),
		    Regex::KleeneStar
		    
		} 
	    }
	}
    }
    return regex
}
*/

fn regex_to_nfa(r:Regex,a:usize) -> NFAForRegex {
    match r {
	Regex::Empty => get_accept_e(a),
	Regex::Single(i) => get_accept_single(i,a),
	Regex::KleeneStar(r) => get_kstar(*r, a),
	Regex::KleenePlus(r) => get_concat((*r).clone(),Regex::KleeneStar(Box::new(*r)),a),
	Regex::QMark(r) => get_or(Regex::Empty,*r,a),
	Regex::Concat((r1,r2)) => get_concat(*r1, *r2,a),
	Regex::Or((r1, r2)) => get_or(*r1,*r2,a)
    }
}
#[derive(Debug)]
struct NFAStateForRegex {
    transitions:Vec<Vec<usize>>,
    accepting:bool
}
#[derive(Debug)]
struct NFAForRegex {
    states:Vec<NFAStateForRegex>,
    starting:usize
}

fn get_accept_e(alphabet_len:usize) -> NFAForRegex {
    let state = NFAStateForRegex {
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
    let start = NFAStateForRegex {
	transitions,
	accepting:false
    };
    let end = NFAStateForRegex {
	transitions:vec![Vec::<usize>::new();alphabet_len],
	accepting:true
    };
    NFAForRegex {
	states:vec![start,end],
	starting: 0
    }
}

fn get_kstar(r:Regex, alphabet_len:usize ) -> NFAForRegex {
    let mut sub =  regex_to_nfa(r,alphabet_len);
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
    sub.states.push(NFAStateForRegex{
	transitions:new_transitions,
	accepting:true
    });
    sub.starting = len;
    return sub;
}

fn get_concat(r1:Regex, r2:Regex, alphabet_len:usize) -> NFAForRegex {
    let mut r1 = regex_to_nfa(r1,alphabet_len);
    let mut r2 = regex_to_nfa(r2, alphabet_len);

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

fn get_or(r1:Regex, r2:Regex, alphabet_len:usize) -> NFAForRegex {
    let mut r1 = regex_to_nfa(r1,alphabet_len);
    let mut r2 = regex_to_nfa(r2, alphabet_len);

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
    r1.states.push(NFAStateForRegex {
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
enum Regex {
    Empty,
    Single(usize),
    KleeneStar(Box<Regex>),
    KleenePlus(Box<Regex>),
    QMark(Box<Regex>),
    Concat((Box<Regex>,Box<Regex>)),
    Or((Box<Regex>,Box<Regex>)),
}
#[derive(Clone,Debug)]
enum InProgress {
    Reg(Regex),
    KStar,
    KPlus,
    QMark,
    Or,
    Open,
    Close
}
