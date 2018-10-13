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

pub use self::token::{Token, TokenType};

// The order of the token mappings in the array matter, to ensure you don't match prematurely.
// e.g. Want "..." to match Ellipsis, not three Dot tokens.
const SIMPLE_TOKENS: [(&str, TokenType); 48] = [
    ("\\\r\n", TokenType::NewlineContinuation),
    ("\\\n", TokenType::NewlineContinuation),
    (">>=", TokenType::RightShiftEqual),
    ("<<=", TokenType::LeftShiftEqual),
    ("//=", TokenType::DoubleSlashEqual),
    ("**=", TokenType::DoubleStarEqual),
    ("...", TokenType::Ellipsis),
    ("!=", TokenType::NotEqual),
    ("&=", TokenType::AmpersandEqual),
    ("@=", TokenType::AtEqual),
    ("^=", TokenType::CircumflexEqual),
    ("%=", TokenType::PercentEqual),
    ("+=", TokenType::PlusEqual),
    ("|=", TokenType::VerticalBarEqual),
    ("==", TokenType::DoubleEqual),
    ("-=", TokenType::MinusEqual),
    ("->", TokenType::RightArrow),
    (">>", TokenType::RightShift),
    (">=", TokenType::GreaterEqual),
    ("<<", TokenType::LeftShift),
    ("<=", TokenType::LessEqual),
    ("//", TokenType::DoubleSlash),
    ("/=", TokenType::SlashEqual),
    ("**", TokenType::DoubleStar),
    ("*=", TokenType::StarEqual),
    (":", TokenType::Colon),
    (",", TokenType::Comma),
    (";", TokenType::Semicolon),
    ("~", TokenType::Tilde),
    ("&", TokenType::Ampersand),
    ("@", TokenType::At),
    ("^", TokenType::Circumflex),
    ("%", TokenType::Percent),
    ("+", TokenType::Plus),
    ("|", TokenType::VerticalBar),
    ("=", TokenType::Equal),
    ("-", TokenType::Minus),
    (">", TokenType::Greater),
    ("<", TokenType::Less),
    ("/", TokenType::Slash),
    ("*", TokenType::Star),
    (".", TokenType::Dot),
    ("(", TokenType::LeftParenthesis),
    ("[", TokenType::LeftSquareBracket),
    ("{", TokenType::LeftBrace),
    (")", TokenType::RightParenthesis),
    ("]", TokenType::RightSquareBracket),
    ("}", TokenType::RightBrace),
];

pub struct Tokenizer<'a> {
    pub source: &'a str,
    /// The position of the current tokenization in bytes.
    pub position: usize,
    pub parentheses_level: i32,
    pub is_finished: bool,
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
        if self.is_finished {
            return None
        }

        let mut characters = self.source[self.position..].chars().peekable();

        // skip whitespace between tokens
        // This peek-predicate-next pattern is required because
        // the borrow checker doesn't support non-lexical borrowing.
        // See: https://stackoverflow.com/a/37509009/9372178
        while let Some(true) = characters.peek().map(|c: &char| {
            *c == ' ' || *c == '\t' || *c == '\x0C'
        }) {
            self.position += 1; // All python whitespace characters are 1 byte.
            characters.next();
        }

        let token =
            match characters.next() {
                Some(c) => match c {
                    '#' => self.comment(),
                    '\r' => match characters.next() {
                        Some(c) if c == '\n' => self.newline("\r\n"),
                        _ => None, // '\r' by itself is not recognized as a newline in Python source
                    },
                    '\n' => self.newline("\n"),
                    // Number tokens must start with [0-9]+ or \.[0-9]+
                    '.' => match characters.next() {
                        Some(c) if c.is_digit(10) => self.number(),
                        _ => None, // Postpone Dot token match for simple token matches
                    },
                    c if c.is_digit(10) => self.number(),
                    c if UnicodeXID::is_xid_start(c) => self.name(),
                    _ => None,
                },
                // This None matches if there are no more characters left in the source.
                None => {
                    self.is_finished = true;
                    Some(Token {
                        token_type: TokenType::EndMarker,
                        value: ""
                    })
                },
            }
            .or_else(|| {
                self::SIMPLE_TOKENS.iter().fold(
                    None,
                    |intermediate_result, simple_token_mapping| match intermediate_result {
                        Some(_) => intermediate_result,
                        None => self.simple(simple_token_mapping.0, simple_token_mapping.1),
                    },
                )
            });

        // Update tokenizer state based on the resultant token
        match &token {
            Some(unwrapped_token) => {
                self.position += unwrapped_token.value.len();
                match unwrapped_token.token_type {
                    TokenType::LeftBrace
                    | TokenType::LeftParenthesis
                    | TokenType::LeftSquareBracket => self.parentheses_level += 1,
                    TokenType::RightBrace
                    | TokenType::RightParenthesis
                    | TokenType::RightSquareBracket => self.parentheses_level -= 1,
                    _ => (),
                }
            },
            None => panic!("TODO: No token matches found! Add appropriate error handling here."),
        };

        token
    }
}

impl<'a> Tokenizer<'a> {
    fn new(source: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            source,
            position: 0,
            parentheses_level: 0,
            is_finished: false,    
        }
    }

    fn simple(&self, value: &'a str, token_type: TokenType) -> Option<Token<'a>> {
        // Bounds check to ensure no panic when slicing to match below.
        if self.source.len() >= self.position + value.len() {
            let candidate_match = &self.source[self.position..(self.position + value.len())];
            if value == candidate_match {
                Some(Token {
                    token_type: token_type,
                    value,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn comment(&self) -> Option<Token<'a>> {
        // Comments continue from any '#' character to a line
        // break, or end of the file.
        // Note, that tokenize.py returns an ErrorToken, whose
        // value is the entire comment if the comment contains
        // a \r with no subsequent \n. This doesn't follow that
        // implementation detail.
        let next_source = &self.source[self.position..];
        if let Some(byte_index) = next_source.find("\r\n") {
            Some(Token {
                token_type: TokenType::Comment,
                value: &self.source[self.position..(self.position + byte_index)],
            })
        } else if let Some(byte_index) = next_source.find("\n") {
            Some(Token {
                token_type: TokenType::Comment,
                value: &self.source[self.position..(self.position + byte_index)],
            })
        } else {
            // In this case, the comment goes until the end of the file with no trailing line break.
            Some(Token {
                token_type: TokenType::Comment,
                value: &self.source[self.position..],
            })
        }
    }

    fn newline(&self, value: &'a str) -> Option<Token<'a>> {
        if self.parentheses_level > 0 {
            Some(Token {
                token_type: TokenType::NewlineContinuation,
                value,
            })
        } else {
            Some(Token {
                token_type: TokenType::NewlineLogical,
                value,
            })
        }
    }

    fn name(&self) -> Option<Token<'a>> {
        let mut characters = self.source[self.position..].char_indices();
        loop {
            if let Some((byte_index, character)) = characters.next() {
                if !UnicodeXID::is_xid_continue(character) {
                    break Some(Token {
                        token_type: TokenType::Name,
                        value: &self.source[self.position..(self.position + byte_index)],
                    });
                }
            } else {
                // This is the case that the name token goes until the end
                // of the file with no trailing line break.
                break Some(Token {
                    token_type: TokenType::Name,
                    value: &self.source[self.position..],
                });
            }
        }
    }

    fn number(&self) -> Option<Token<'a>> {
        None
    }
}

pub fn tokenize(source: &str) -> Vec<Token> {
    Tokenizer::new(source).collect()
}

// pub fn untokenize(tokens: Vec<Token>) -> &str {
//     "" // TODO: Implement function body
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy() {
        let mut tokenizer = Tokenizer::new("def some_func():\n    x = some_var #this is a comment\n    return x");
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.next());
        
        assert!(true);
    }

    #[test]
    fn ops() {
        let mut tokenizer = Tokenizer::new("def some_func():");
        println!("{:?}", tokenizer.position);
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.position);
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.position);
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.position);
        println!("{:?}", tokenizer.next());
        println!("{:?}", tokenizer.position);
        println!("{:?}", tokenizer.next());
        
        assert!(true);
    }

    #[test]
    fn simple_token_three_chars_exact_match() {
        let tokenizer = Tokenizer::new(">>=");
        let expected = Some(Token {
            token_type: TokenType::RightShiftEqual,
            value: ">>=",
        });
        let actual = tokenizer.simple(">>=", TokenType::RightShiftEqual);
        assert_eq!(expected, actual);
    }

    #[test]
    fn simple_token_two_chars_exact_match() {
        let tokenizer = Tokenizer::new(">>");
        let expected = Some(Token {
            token_type: TokenType::RightShift,
            value: ">>",
        });
        let actual = tokenizer.simple(">>", TokenType::RightShift);
        assert_eq!(expected, actual);
    }

    #[test]
    fn simple_token_one_char_exact_match() {
        let tokenizer = Tokenizer::new(">");
        let expected = Some(Token {
            token_type: TokenType::Greater,
            value: ">",
        });
        let actual = tokenizer.simple(">", TokenType::Greater);
        assert_eq!(expected, actual);
    }

    #[test]
    fn simple_token_val_length_greater_than_source() {
        let tokenizer = Tokenizer::new(">>");
        let expected = None;
        let actual = tokenizer.simple(">>=", TokenType::RightShiftEqual);
        assert_eq!(expected, actual);
    }

    #[test]
    fn ampersand_with_more_characters() {
        let mut tokenizer = Tokenizer::new("&abcd");
        let expected = Some(Token {
            token_type: TokenType::Ampersand,
            value: "&",
        });
        let actual = tokenizer.next();
        assert_eq!(expected, actual);
    }

    #[test]
    fn ampersand_last_character_in_source() {
        let mut tokenizer = Tokenizer::new("&");
        let expected = Some(Token {
            token_type: TokenType::Ampersand,
            value: "&",
        });
        let actual = tokenizer.next();
        assert_eq!(expected, actual);
    }

    #[test]
    fn ampersand_equal_with_more_characters() {
        let mut tokenizer = Tokenizer::new("&=abcd");
        let expected = Some(Token {
            token_type: TokenType::AmpersandEqual,
            value: "&=",
        });
        let actual = tokenizer.next();
        assert_eq!(expected, actual);
    }

    #[test]
    fn ampersand_equal_last_character_in_source() {
        let mut tokenizer = Tokenizer::new("&=");
        let expected = Some(Token {
            token_type: TokenType::AmpersandEqual,
            value: "&=",
        });
        let actual = tokenizer.next();
        assert_eq!(expected, actual);
    }

    /*
    #[test]
    fn fraction_entire_string_is_token_2_chars() {
        let expected = Token {
            token_type: TokenType::Number,
            value: ".1",
        };
        let actual = fraction(".1");
        assert_eq!(expected, actual);
    }
    
    #[test]
    fn fraction_entire_string_is_token_many_chars_no_underscores() {
        let expected = Token {
            token_type: TokenType::Number,
            value: ".1789078875675",
        };
        let actual = fraction(".1789078875675");
        assert_eq!(expected, actual);
    }
    
    #[test]
    fn fraction_entire_string_is_token_with_underscores() {
        let expected = Token {
            token_type: TokenType::Number,
            value: ".178_907_887_567_5",
        };
        let actual = fraction(".178_907_887_567_5");
        assert_eq!(expected, actual);
    }
    
    #[test]
    fn fraction_only_part_of_string_is_token() {
        let expected = Token {
            token_type: TokenType::Number,
            value: ".1",
        };
        let actual = fraction(".1abcd123");
        assert_eq!(expected, actual);
    }
    
    #[test]
    fn fraction_with_dangling_underscore() {
        let expected = Token {
            token_type: TokenType::Number,
            value: ".178_907_887_567",
        };
        let actual = fraction(".178_907_887_567_");
        assert_eq!(expected, actual);
    }
    */
}
