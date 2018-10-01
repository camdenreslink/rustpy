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
//! This tokenizer is implemented as an LL(1) recursive descent parser [^language-implementation-patterns-parr]. One unique wrinkle in Python's tokenizer, is that it also handles some of the logic of significant whitespace specifying code blocks through the `INDENT` and `DEDENT` tokens.
//! 
//! Notes/Considerations
//! --------------------
//! 
//! * As of Python 3.7.0 UTF-16 is not supported as a source encoding.
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
//! * [tokenizer.c](https://github.com/python/cpython/blob/master/Parser/tokenizer.c) - Parsing loop is in `tok_get` function.
//! * [tokenizer.h](https://github.com/python/cpython/blob/master/Parser/tokenizer.h)
//! 
//! Additional Resources
//! --------------------
//! 
//! * [Unicode Literals in Python Source Code](https://docs.python.org/3/howto/unicode.html#unicode-literals-in-python-source-code)
//! * [The Guts of Unicode in Python - Video](https://pyvideo.org/pycon-us-2013/the-guts-of-unicode-in-python.html)
//!  
//! [^language-implementation-patterns-parr]: [Language Implementation Patterns](https://pragprog.com/book/tpdsl/language-implementation-patterns) by Terence Parr; Pattern 2: LL(1) Recursive-Descent Lexer
pub fn tokenize() {
    ()
}