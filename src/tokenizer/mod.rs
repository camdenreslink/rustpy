//! Summary
//! -------
//! 
//! This module is responsible for tokenizing Python source code.
//! 
//! What is Tokenizing?
//! -------------------
//! 
//! Tokenizing is the process of transforming a sequence of characters into a predefined set of tokens which have special meaning in a programming language.
//! 
//! Design/Implementation
//! ---------------------
//! 
//! This tokenizer is implemented as an LL(1) recursive descent parser. [^language-implementation-patterns-parr]
//! 
//! Relevant PEPs
//! -------------
//! 
//! * [PEP 263](https://www.python.org/dev/peps/pep-0263/) - Defining Python Source Code Encodings
//! * [PEP 3120](https://www.python.org/dev/peps/pep-3120/) - Using UTF-8 as the default source encoding
//! 
//! Links to CPython Source Code
//! ----------------------------
//! 
//! * [tokenizer.c](https://github.com/python/cpython/blob/master/Parser/tokenizer.c)
//! * [tokenizer.h](https://github.com/python/cpython/blob/master/Parser/tokenizer.h)
//! 
//! [^language-implementation-patterns-parr]: [Language Implementation Patterns](https://pragprog.com/book/tpdsl/language-implementation-patterns) by Terence Parr; Pattern 2: LL(1) Recursive-Descent Lexer
pub fn tokenize() {
    ()
}