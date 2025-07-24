use crate::regex::RegexTree;
use crate::regex::Regex;
use crate::nfa::NFA;
use crate::nfa::NFAState;

pub fn nfa_to_regex(nfa:&NFA) -> Regex {
	let alphabet:String=nfa.alphabet.clone();
	let mut table = get_2d_array(nfa);
	for i in 0..(table.len()-1) {
		bypass_state(&mut table,i);
	}
	let rt = table.last().unwrap().last().unwrap();
	return Regex::new(alphabet,rt.clone());
}

fn bypass_state(table:&mut Vec<Vec<Option<RegexTree>>>,line:usize) {
	//all lines before this one have already been removed so can be ignored.

	let self_loop = table[line][line].clone();
	let leaving = get_leaving(table[line].clone(),line);

	for state in (line+1)..table.len() {
		if let Some(original) = &table[state][line] {
			//does go to state we're removing
			let start:RegexTree = match &self_loop {
				Some(l) => RegexTree::Concat((Box::new(original.clone()),Box::new(RegexTree::KleeneStar(Box::new(l.clone()))))),
				None => original.clone()
			};
			for (way,destination) in &leaving {
				table[state][*destination] = match &table[state][*destination] {
					None => Some(
						RegexTree::Concat((Box::new(start.clone()),Box::new((*way).clone())))),
					Some(rt) => Some(
							RegexTree::Or((
								Box::new(rt.clone()),
								Box::new(RegexTree::Concat((Box::new(start.clone()),Box::new((*way).clone()))))
							))
					)
				}
			}
			
			// just to be sure
			table[state][line]=None;
			
		}
	}
}

fn get_leaving(line:Vec<Option<RegexTree>>,num:usize) -> Vec<(RegexTree,usize)> {
	let mut result = Vec::new();
	for i in 0..line.len() {
		if i!=num {
			if let Some(rt) = &line[i] {
				result.push((rt.clone(),i));
			}
		} 
	}

	return result;
}

fn get_2d_array(nfa:&NFA) -> Vec<Vec<Option<RegexTree>>> {
	let size = nfa.states.len() +2;

	let mut new_start:Vec<Option<RegexTree>> = vec![None;size];
	new_start[nfa.starting]=Some(RegexTree::Empty);

	let mut result:Vec<Vec<Option<RegexTree>>> = (&nfa.states).into_iter().map(|s| get_1d_array(&s,size)).collect();
	result.push(new_start);
	return result;
}

fn get_1d_array(state:&NFAState,size:usize) -> Vec<Option<RegexTree>> {
	let mut result:Vec<Option<RegexTree>> = vec![None;size];
	if state.accepting {
		result[size-1] = Some(RegexTree::Empty);
	}
	let num_letters = state.transitions.len();

	let letter = 0;
	for next in &state.transitions[letter] {
		result[*next] = match &result[*next] {
			None => Some(RegexTree::Empty),
			Some(rt) => Some(
				RegexTree::Or((
					Box::new(rt.clone()),
					Box::new(RegexTree::Empty)
				))
			)
		};
	}
	
	
	for letter in 1..num_letters {
		for next in &state.transitions[letter] {
			result[*next] = match &result[*next] {
				None => Some(RegexTree::Single(letter-1)),
				Some(rt) => Some(
					RegexTree::Or((
						Box::new(rt.clone()),
						Box::new(RegexTree::Single(letter-1))
					))
				)
			};
		}
	}	
	return result;
}
