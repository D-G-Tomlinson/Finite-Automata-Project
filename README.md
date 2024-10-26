# complexity-exercise-1-4

This program takes in 
- A Deterministic Finite Automata (DFA) and a word and computes whether that word is accepted
- A Nondeterministic Finite Automata (NFA) and at least one of:
  - An output .dfa file to convert the NFA to
  - A word to be accepted or rejected by the NFA
 
 The structure of a .dfa file m states is:
 - First line is the alphabet, the letter e is not allowed
 - Second line is the starting state number (indexed from 1)
 - Lines 3 through m+2 contain a comma-seperated-list of the state (indexed from 1) reached from this state, by the corresponding letter of the alphabet, followed by "true" or "false" depending on if the state is accepting
 
 The structure of a .nfa file with m states is
 - First line is the alphabet, the letter e is not allowed
 - Second line is the starting state number (indexed from 1)
 - Lines 3 through m+2 contain a comma-seperated-list of arrows out of that state, in the format letter:next-state-number, followed by "true" or "false" depending on if the state is accepting
   - The number of letter:next-state-number pairs is unrestricted, as NFAs can have multiple arrows with the same letter from the same state, or no arrows at all. The letter e, representing a jump, is allowed
