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

        // Mutability required to call `is_xid_start` on a character
        let mut characters = next_source.chars().peekable();

        match characters.next() {
            Some(c) => match c {
                ' ' | '\t' | '\x0C' => self.next(),
                '#' => Some(token_generators::comment(next_source)),
                ':' => Some(Token {
                    token_type: TokenType::Colon,
                    value: ":",
                }),
                ',' => Some(Token {
                    token_type: TokenType::Comma,
                    value: ",",
                }),
                '[' => Some(Token {
                    token_type: TokenType::LeftSquareBracket,
                    value: "[",
                }),
                '{' => Some(Token {
                    token_type: TokenType::LeftBrace,
                    value: "{",
                }),
                '(' => Some(Token {
                    token_type: TokenType::LeftParenthesis,
                    value: "(",
                }),
                ']' => Some(Token {
                    token_type: TokenType::RightSquareBracket,
                    value: "]",
                }),
                '}' => Some(Token {
                    token_type: TokenType::RightBrace,
                    value: "}",
                }),
                ')' => Some(Token {
                    token_type: TokenType::RightParenthesis,
                    value: ")",
                }),
                ';' => Some(Token {
                    token_type: TokenType::Semicolon,
                    value: ";",
                }),
                '~' => Some(Token {
                    token_type: TokenType::Tilde,
                    value: "~",
                }),
                '!' => match characters.peek() {
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::NotEqual,
                            value: "!=",
                        })
                    }
                    _ => panic!("TODO: Add actual error handling."),
                },
                '&' => match characters.peek() {
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::AmpersandEqual,
                            value: "&=",
                        })
                    }
                    _ => Some(Token {
                        token_type: TokenType::Ampersand,
                        value: "&",
                    }),
                },
                '@' => match characters.peek() {
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::AtEqual,
                            value: "@=",
                        })
                    }
                    _ => Some(Token {
                        token_type: TokenType::At,
                        value: "@",
                    }),
                },
                '^' => match characters.peek() {
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::CircumflexEqual,
                            value: "^=",
                        })
                    }
                    _ => Some(Token {
                        token_type: TokenType::Circumflex,
                        value: "^",
                    }),
                },
                '%' => match characters.peek() {
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::PercentEqual,
                            value: "%=",
                        })
                    }
                    _ => Some(Token {
                        token_type: TokenType::Percent,
                        value: "%",
                    }),
                },
                '+' => match characters.peek() {
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::PlusEqual,
                            value: "+=",
                        })
                    }
                    _ => Some(Token {
                        token_type: TokenType::Plus,
                        value: "+",
                    }),
                },
                '|' => match characters.peek() {
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::VerticalBarEqual,
                            value: "|=",
                        })
                    }
                    _ => Some(Token {
                        token_type: TokenType::VerticalBar,
                        value: "|",
                    }),
                },
                '=' => match characters.peek() {
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::DoubleEqual,
                            value: "==",
                        })
                    }
                    _ => Some(Token {
                        token_type: TokenType::Equal,
                        value: "=",
                    }),
                },
                '-' => match characters.peek() {
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::MinusEqual,
                            value: "-=",
                        })
                    }
                    Some('>') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::RightArrow,
                            value: "->",
                        })
                    }
                    _ => Some(Token {
                        token_type: TokenType::Minus,
                        value: "-",
                    }),
                },
                '>' => match characters.peek() {
                    Some('>') => {
                        characters.next();
                        match characters.peek() {
                            Some('=') => {
                                characters.next();
                                Some(Token {
                                    token_type: TokenType::RightShiftEqual,
                                    value: ">>=",
                                })
                            }
                            _ => Some(Token {
                                token_type: TokenType::RightShift,
                                value: ">>",
                            }),
                        }
                    }
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::GreaterEqual,
                            value: ">=",
                        })
                    }
                    _ => Some(Token {
                        token_type: TokenType::Greater,
                        value: ">",
                    }),
                },
                '<' => match characters.peek() {
                    Some('<') => {
                        characters.next();
                        match characters.peek() {
                            Some('=') => {
                                characters.next();
                                Some(Token {
                                    token_type: TokenType::LeftShiftEqual,
                                    value: "<<=",
                                })
                            }
                            _ => Some(Token {
                                token_type: TokenType::LeftShift,
                                value: "<<",
                            }),
                        }
                    }
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::LessEqual,
                            value: "<=",
                        })
                    }
                    _ => Some(Token {
                        token_type: TokenType::Less,
                        value: "<",
                    }),
                },
                '/' => match characters.peek() {
                    Some('/') => {
                        characters.next();
                        match characters.peek() {
                            Some('=') => {
                                characters.next();
                                Some(Token {
                                    token_type: TokenType::DoubleSlashEqual,
                                    value: "//=",
                                })
                            }
                            _ => Some(Token {
                                token_type: TokenType::DoubleSlash,
                                value: "//",
                            }),
                        }
                    }
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::SlashEqual,
                            value: "/=",
                        })
                    }
                    _ => Some(Token {
                        token_type: TokenType::Slash,
                        value: "/",
                    }),
                },
                '*' => match characters.peek() {
                    Some('*') => {
                        characters.next();
                        match characters.peek() {
                            Some('=') => {
                                characters.next();
                                Some(Token {
                                    token_type: TokenType::DoubleStarEqual,
                                    value: "**=",
                                })
                            }
                            _ => Some(Token {
                                token_type: TokenType::DoubleStar,
                                value: "**",
                            }),
                        }
                    }
                    Some('=') => {
                        characters.next();
                        Some(Token {
                            token_type: TokenType::StarEqual,
                            value: "*=",
                        })
                    }
                    _ => Some(Token {
                        token_type: TokenType::Star,
                        value: "*",
                    }),
                },
                '.' => match characters.peek() {
                    Some('.') => {
                        characters.next();
                        match characters.peek() {
                            Some('.') => {
                                characters.next();
                                Some(Token {
                                    token_type: TokenType::Ellipsis,
                                    value: "...",
                                })
                            }
                            _ => panic!("TODO: Add actual error handling."),
                        }
                    }
                    Some(&c) if c.is_digit(10) => {
                        characters.next();
                        Some(token_generators::number(next_source))
                    }
                    _ => Some(Token {
                        token_type: TokenType::Dot,
                        value: ".",
                    }),
                },
                c if c.is_digit(10) => Some(token_generators::number(next_source)),
                c if UnicodeXID::is_xid_start(c) => Some(token_generators::name(next_source)),
                _ => panic!("TODO: No character match! Add proper error handling!"),
            },
            None => Some(Token {
                token_type: TokenType::EndMarker,
                value: "",
            }),
        }
    }
}

// Not sure how I want to organize this token matching code

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

// pub fn untokenize(tokens: Vec<Token>) -> &str {
//     "" // TODO: Implement function body
// }

#[cfg(test)]
mod tests {
    use super::*;

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
}
