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
//! * [https://docs.python.org/3/library/token.html](https://docs.python.org/3/library/token.html)
//! 
//! Additional Resources
//! --------------------
//! 
//! * [Unicode Literals in Python Source Code](https://docs.python.org/3/howto/unicode.html#unicode-literals-in-python-source-code)
//! * [The Guts of Unicode in Python - Video](https://pyvideo.org/pycon-us-2013/the-guts-of-unicode-in-python.html)
//! * 
//!  
//! [^language-implementation-patterns-parr]: [Language Implementation Patterns](https://pragprog.com/book/tpdsl/language-implementation-patterns) by Terence Parr; Pattern 2: LL(1) Recursive-Descent Lexer

pub enum TokenType {
    Ampersand,
    AmpersandEqual,
    At,
    AtEqual,
    Circumflex,
    CircumflexEqual,
    Colon,
    Comma,
    Comment,
    Dedent,
    Dot,
    DoubleEqual,
    DoubleSlash,
    DoubleSlashEqual,
    DoubleStar,
    DoubleStarEqual,
    Ellipsis,
    Encoding,
    EndMarker,
    Equal,
    ErrorToken,
    Greater,
    GreaterEqual,
    Indent,
    LeftSquareBracket,
    LeftBrace,
    LeftParenthesis,
    LeftShift,
    LeftShiftEqual,
    Less,
    LessEqual,
    MinusEqual,
    Minus,
    Name,
    NewlineContinuation,
    NewlineLogical,
    NotEqual,
    Number,
    Operator,
    Percent,
    PercentEqual,
    Plus,
    PlusEqual,
    RightArrow,
    RightBrace,
    RParenthesis,
    RightSquareBracket,
    RightShift,
    RightShiftEqual,
    Semicolon,
    Slash,
    SlashEqual,
    Star,
    StarEqual,
    String,
    Tilde,
    VerticalBar,
    VerticalBarEqual
}

pub struct Token<'a> {
    token_type: TokenType,
    value: &'a str,
}

pub fn tokenize(source: &str) -> Vec<Token> {
    for character in source.chars() {
        
    }
    vec![]
}

pub fn untokenize(tokens: Vec<Token>) -> &str {
    ""
}