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
//! * This tokenizer is implemented as an LL(1) recursive descent parser [^language-implementation-patterns-parr].
//! * One unique wrinkle in Python's tokenizer, is that it also handles some of the logic of significant whitespace specifying code blocks through the `INDENT` and `DEDENT` tokens.
//! * This implementation has changed the names of the tokens to be Pascal-case (as opposed to the all caps in the CPython implementation), to conform with rust naming conventions for enum variants. Abbreviations in the token names have now been expanded to fully spelled out words.
//!
//! Notes/Considerations
//! --------------------
//!
//! * As of Python 3.7.0 UTF-16 is not supported as a source encoding.
//! * Python identifiers follow
//! * Python supports Unicode version 11.0.0, which can be found by running `import unicodedata` then `unicodedata.unidata_version` at a Python interactive prompt.
//!
//! Relevant PEPs
//! -------------
//!
//! * [PEP 263](https://www.python.org/dev/peps/pep-0263/) - Defining Python Source Code Encodings
//! * [PEP 3120](https://www.python.org/dev/peps/pep-3120/) - Using UTF-8 as the default source encoding
//! * [PEP 3131](https://www.python.org/dev/peps/pep-3131/) - Supporting Non-ASCII Identifiers
//!
//! CPython Source Code
//! -------------------
//!
//! * [tokenizer.c](https://github.com/python/cpython/blob/master/Parser/tokenizer.c) - Parsing loop is in `tok_get` function.
//! * [tokenizer.h](https://github.com/python/cpython/blob/master/Parser/tokenizer.h)
//! * [token.h](https://github.com/python/cpython/blob/master/Include/token.h)
//! * [tokenize.py](https://github.com/python/cpython/blob/master/Lib/tokenize.py)
//! * [token.py](https://github.com/python/cpython/blob/master/Lib/token.py)
//!
//! CPython Documentation
//! ---------------------
//!
//! * [https://docs.python.org/3/reference/lexical_analysis.html](https://docs.python.org/3/reference/lexical_analysis.html)
//! * [https://docs.python.org/3/library/token.html](https://docs.python.org/3/library/token.html)
//! * [Unicode Literals in Python Source Code](https://docs.python.org/3/howto/unicode.html#unicode-literals-in-python-source-code)
//!
//! Additional Resources
//! --------------------
//!
//! * [The Guts of Unicode in Python - Video](https://pyvideo.org/pycon-us-2013/the-guts-of-unicode-in-python.html)
//! *
//!
//! [^language-implementation-patterns-parr]: [Language Implementation Patterns](https://pragprog.com/book/tpdsl/language-implementation-patterns) by Terence Parr; Pattern 2: LL(1) Recursive-Descent Lexer

use unicode_xid::UnicodeXID;

pub mod token;
pub mod token_generators;

pub use self::token::{Token, TokenType};

pub struct Tokenizer<'a> {
    pub source: &'a str,
    /// The position of the current tokenization in bytes.
    pub position: usize,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    // There seem to actually be 2 scenarios when tokenizing,
    // the beginning of a new logical line (which tokenizes identation)
    // and any other type of token. How do we know if we aren't in a
    // continuation line, or a legal multiline logical line (e.g. within
    // parens). We need to keep track of a stack of brackets, braces, and parens,
    // and look back one token to see if the previous token was a NewlineContinuation
    // or NewlineLogical. So, not a true LL(1) recursive descent parser.
    fn next(&mut self) -> Option<Self::Item> {
        let next_source = &self.source[self.position..];
        let characters: Vec<char> = next_source.chars().take(3).collect();

        match characters.as_slice() {
            [c1, _, _] if ignore_whitespace(c1) => self.next(),
            ['#', _, _] => Some(token_generators::comment(next_source)),
            [c1, _, _] if UnicodeXID::is_xid_start(*c1) => Some(token_generators::name(next_source)),
            _ => None, //TODO: Add proper error handling!
        }
    }
}

// Not sure how I want to organize this token matching code
fn ignore_whitespace(c: &char) -> bool {
    match c {
        ' ' | '\t' | '\x0C' => true,
        _ => false,
    }
}
// End matching code section - TODO: Decide how to organize

impl<'a> Tokenizer<'a> {
    fn new(source: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            source,
            position: 0,
        }
    }
}

pub fn tokenize(source: &str) -> Vec<Token> {
    Tokenizer::new(source).collect()
}

pub fn untokenize(tokens: Vec<Token>) -> &str {
    "" // TODO: Implement function body
}
