# Finite Automata Project

## Input

Note: the following characters are not permitted to be used as individual letters in alphabets etc., as they are special characters for writing DFAs, NFAs and regexes:
 - ,
 - :
 - (
 - )
 - |
 - \*
 - \+
 - ?

This program takes in one of: 
- The address of a .dfa file containing a Deterministic Finite Automata (DFA) with the option --input.
- The address of a .nfa file containing a Non-deterministic Finite Automata (NFA) with the option --input.
- A regex as a string with the option --regex.

### DFA

 The structure of a .dfa file with m states is:
 - The first line is the alphabet.
 - The second line is the starting state number (indexed from 1).
 - Lines 3 through m+2 (inclusive) contain a comma-seperated-list of the state (indexed from 1) reached from this state, by the corresponding letter of the alphabet, followed by "true" or "false" depending on if the state is accepting.

### NFA

 The structure of a .nfa file with m states is:
 - The first line is the alphabet.
 - The second line is the starting state number (indexed from 1).
 - Lines 3 through m+2 contain a comma-seperated-list of arrows out of that state, in the format _letter:next-state-number_, followed by "true" or "false" depending on if the state is accepting
   - The number of letter:next-state-number pairs is unrestricted, as NFAs can have multiple arrows with the same letter from the same state, or no arrows at all. To represent a jump, have no alphabet character preceeding the ":".

### Regex

The regex understood by the program consists of operators and base characters. The operators, in order of priority, are as follows:
- () used to control the order of operation.
- *, +, ? unary postfix operators. * represents any number of the preceeding object, + represents any positive number of the preceeding object, ? is zero or one of the proceeding object.
- | infix operator, either the left object or the right object would be accepted.

All other permissible characters will be understood to represent a word containing only itself.

## Output

The program can produce any of the following output, including multiple options in the same execution of the program:
- Determining if a given word is in ("ACCEPTED") or not in ("REJECTED") the language described by the input, with the option --word.
- A DFA equivalent to the input, written to a .dfa file specified with --dfa-output.
- An NFA equivalent to the input, written to a .nfa file specified with --nfa-output.
- A regex equivalent to the input, printed to the standard output, with the flag --regex-output

## Example Use

For example the command

`cargo run -- --regex '(a|b)*ab' --dfa-output Endab.dfa --nfa-output Endab.nfa`

Will convert the given regex (accepting all words containing only a and b and ending with ab) to an equivalent NFA and DFA. Note, due to the nature of the algorithm, the resultant NFA contains more states than necessary - it is not the simplest form.
