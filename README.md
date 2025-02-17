# Finite Automata Project

This program takes in one of: 
- A Deterministic Finite Automata (DFA) and a word and computes whether that word is accepted
- A Nondeterministic Finite Automata (NFA) and at least one of:
  - An output .dfa file to convert the NFA to
  - A word to be accepted or rejected by the NFA
 
 The structure of a .dfa file with m states is:
 - First line is the alphabet
 - Second line is the starting state number (indexed from 1)
 - Lines 3 through m+2 contain a comma-seperated-list of the state (indexed from 1) reached from this state, by the corresponding letter of the alphabet, followed by "true" or "false" depending on if the state is accepting
 
 The structure of a .nfa file with m states is
 - First line is the alphabet
 - Second line is the starting state number (indexed from 1)
 - Lines 3 through m+2 contain a comma-seperated-list of arrows out of that state, in the format letter:next-state-number, followed by "true" or "false" depending on if the state is accepting
   - The number of letter:next-state-number pairs is unrestricted, as NFAs can have multiple arrows with the same letter from the same state, or no arrows at all. To represent a jump, have no alphabet character preceeding the : 

For example the command

`cargo run -- --regex '(a|b)*ab' --dfa-output Endab.dfa --nfa-output Endab.nfa`

Will convert the given regex (accepting all words containing only a and b and ending with ab) to an NFA and a DFA (equivalent to Exc1b.nfa). Note, due to the nature of the algorithm, the resultant NFA contains more states than necessary - it is not the simplest form.