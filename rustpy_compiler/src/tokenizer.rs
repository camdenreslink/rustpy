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
//! * This implementation has changed the names of the tokens to be Pascal-case (as opposed to the all caps in the CPython implementation), to conform with rust naming conventions for enum variants. Abbreviations in the token names have been expanded to fully spelled out words.
//!
//! Notes/Considerations
//! --------------------
//!
//! * As of Python 3.7.0 UTF-16 is not supported as a source encoding.
//! * Python identifiers follow
//! * Python supports Unicode version 11.0.0, which can be found by running `import unicodedata` then `unicodedata.unidata_version` at a Python interactive prompt.
//! * The maximum level of indentation is hard coded to 100 in CPython.
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
//! CPython Documentation/Bugs
//! --------------------------
//!
//! * [https://docs.python.org/3/reference/lexical_analysis.html](https://docs.python.org/3/reference/lexical_analysis.html)
//! * [https://docs.python.org/3/library/token.html](https://docs.python.org/3/library/token.html)
//! * [Unicode Literals in Python Source Code](https://docs.python.org/3/howto/unicode.html#unicode-literals-in-python-source-code)
//! * [Bug: tokenize module happily tokenizes code with syntax errors](https://bugs.python.org/issue12675) - The tokenize module has totally different error handling logic than tokenizer.c (actually used by Python).
//! * [Bug: TabError behavior doesn't match documentation](https://bugs.python.org/issue24260) - The Python 3 indentation logic is poorly/inconsistently documented. You aren't supposed to mix tabs/spaces, but sometimes it lets you. Can't find the scenarios where TabError is raised documented anywhere.
//!
//! Additional Resources
//! --------------------
//!
//! * [The Guts of Unicode in Python - Video](https://pyvideo.org/pycon-us-2013/the-guts-of-unicode-in-python.html)
//! * [Python 3 allows mixing spaces and tabs?](https://stackoverflow.com/a/36064673/9372178)
//! * [UAX #31 - Unicode Identifier and Pattern Syntax](https://www.unicode.org/reports/tr31/tr31-29.html)
//!
//! [^language-implementation-patterns-parr]: [Language Implementation Patterns](https://pragprog.com/book/tpdsl/language-implementation-patterns) by Terence Parr; Pattern 2: LL(1) Recursive-Descent Lexer

use std::collections::VecDeque;
use unicode_normalization::UnicodeNormalization;
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

const TAB_SIZE: u32 = 8;

pub struct Tokenizer<'a> {
    /// The source string to be tokenized. A precondition of the tokenizer is that the source has been validated from an encoding perspective and converted to utf-8.
    pub source: &'a str,
    /// The position (cursor) of the current tokenization in bytes.
    pub position: usize,
    pub parentheses_level: i32,
    /// Previous token type must be tracked, because it might affect which token gets generated. E.g. whether whitespace should be considered indentation vs. ignored intertoken spacing. This depends on if the previous token was/wasn't a NewlineLogical token.
    pub previous_token_type: Option<TokenType>,
    pub indentation_stack: Vec<&'a str>,
    /// If the next() tokenization generates multiple tokens, (for example a single newline ending multiple block scopes which generates multiple Dedent tokens) we can only return one. This token_buffer will store the others, and return them on subsequent calls to next().
    pub token_buffer: VecDeque<Token>,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token; // TODO: Change this to Result<Token, TokenError> to handle errors idiomatically (vs ErrorTokens used in CPython)

    fn next(&mut self) -> Option<Self::Item> {
        // Return early if we've already completed the tokenization.
        if self.position >= self.source.len() {
            return None;
        }

        let buffered_token = self.token_buffer.pop_front();

        let token = buffered_token
            .or_else(|| {
                // If this is the beginning of a logical line, calculate the indent/dedent tokens (if any)
                match self.previous_token_type {
                    Some(TokenType::NewlineLogical) => {
                        self.token_buffer = self.indentation();
                        self.token_buffer.pop_front()
                    }
                    _ => {
                        // skip insignificant whitespace between tokens
                        self.position += self.whitespace_bytes();
                        None
                    },
                }
            })
            .or_else(|| {
                let mut characters = self.source[self.position..].chars();

                match characters.next()? {
                    '#' => self.comment(),
                    '\r' => match characters.next()? {
                        '\n' => self.newline("\r\n"),
                        _ => None, // '\r' by itself is not recognized as a newline in Python source
                    },
                    '\n' => self.newline("\n"),
                    // Number tokens must start with [0-9]+ or \.[0-9]+
                    '.' => match characters.next()? {
                        c if c.is_digit(10) => self.number(),
                        _ => None, // Postpone Dot token match for simple token matches
                    },
                    c if c.is_digit(10) => self.number(),
                    c if UnicodeXID::is_xid_start(c) => self.name(),
                    _ => None,
                }
            })
            .or_else(|| {
                // Iterate through the simple tokens until a match is found.
                for simple_token_mapping in self::SIMPLE_TOKENS.iter() {
                    let simple_token = self.simple(simple_token_mapping.0, simple_token_mapping.1);
                    if simple_token.is_some() {
                        return simple_token;
                    }
                }

                None
            });

        match &token {
            // Token was successfully matched! Update tokenizer state based on the resultant token.
            Some(unwrapped_token) => {
                self.update(unwrapped_token);
            }
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
            previous_token_type: None,
            indentation_stack: Vec::new(),
            token_buffer: VecDeque::new(),
        }
    }

    fn update(&mut self, token: &Token) {
        self.position += token.value.len();
        match token.token_type {
            TokenType::LeftBrace
            | TokenType::LeftParenthesis
            | TokenType::LeftSquareBracket => self.parentheses_level += 1,
            TokenType::RightBrace
            | TokenType::RightParenthesis
            | TokenType::RightSquareBracket => self.parentheses_level -= 1,
            // TODO: Match on indent/dedent, and add/remove from self.indentation_stack (must take diff of current value and only append the extra bits at the end for an indent)
            _ => (),
        };
        self.previous_token_type = Some(token.token_type);
    }

    fn is_whitespace(character: char) -> bool {
        match character {
            ' ' | '\t' | '\x0c' => true,
            _ => false,
        }
    }

    fn indentation_level(indentation: &str) -> u32 {
        indentation.chars().fold(0, |acc, c| {
            match c {
                ' ' => acc + 1,
                '\t' => ((acc / self::TAB_SIZE) + 1) * self::TAB_SIZE, // Note: / is floor division here
                // Choosing to not have form feed affect indentation level at all, diverging from CPython which resets the indentation level to zero when a form feed is encountered.
                '\x0c' => acc,
                _ => panic!(
                    "Encountered illegal characters while trying to calculate indentation level! Only whitespace characters ' ', '\\t' and '\\x0c' (form feed) are allowed."
                ),
            }
        })
    }

    fn whitespace_bytes(&self) -> usize {
        let mut number_of_bytes: usize = 0;

        let mut characters = self.source[self.position..].chars();
        while let Some(character) = characters.next() {
            if Tokenizer::is_whitespace(character) {
                number_of_bytes += character.len_utf8();
            } else {
                break;
            }
        }

        number_of_bytes
    }

    fn indentation(&self) -> VecDeque<Token> {
        let mut token_buffer = VecDeque::new();
        let mut indentation_value = String::from("");

        let mut characters = self.source[self.position..].chars().peekable();
        while let Some(true) = characters
            .peek()
            .map(|character: &char| Tokenizer::is_whitespace(*character))
        {
            // We know .next() will return Some because we just peeked above.
            if let Some(character) = characters.next() {
                indentation_value.push(character);
            }
        }

        let is_line_all_comment_or_whitespace = match characters.next() {
            Some(character) => match character {
                '#' | '\n' => true,
                '\r' => match characters.next() {
                    Some(character) if character == '\n' => true,
                    _ => false, // '\r' by itself is not recognized as a newline in Python source
                },
                _ => false,
            },
            None => true, // Final line in source string, all whitespace
        };

        // Indent/Dedent tokens shouldn't be generated if there is no indentation, or if the entire line is whitespace/comment. Return early.
        if indentation_value.is_empty() || is_line_all_comment_or_whitespace {
            return token_buffer; // empty at this point
        }

        let current_indentation_value = self.indentation_stack.join("");
        let current_indentation_level = Tokenizer::indentation_level(&current_indentation_value);
        let new_indentation_level = Tokenizer::indentation_level(&indentation_value);

        if new_indentation_level == current_indentation_level {
            if !indentation_value.starts_with(&current_indentation_value) {
                panic!("TabError - Using an inconsistent mix of tabs and spaces!");
            }
        } else if new_indentation_level > current_indentation_level {
            if !indentation_value.starts_with(&current_indentation_value) {
                panic!("TabError - Using an inconsistent mix of tabs and spaces!");
            }

            token_buffer.push_front(Token {
                token_type: TokenType::Indent,
                value: indentation_value,
            });
        } else {
            // new_indentation_level < current_indentation_level
            if !current_indentation_value.starts_with(&indentation_value) {
                panic!("TabError - Using an inconsistent mix of tabs and spaces!");
            }
            // TODO: Add dedents to token_buffer here
        }

        token_buffer
    }

    fn simple(&self, value: &'a str, token_type: TokenType) -> Option<Token> {
        // Bounds check to ensure no panic when slicing to match below.
        if self.source.len() >= self.position + value.len() {
            let candidate_match = &self.source[self.position..(self.position + value.len())];
            if value == candidate_match {
                Some(Token {
                    token_type: token_type,
                    value: String::from(value),
                })
            } else {
                None
            }
        } else {
            // Trying to match a simple token value past the end of the source string
            None
        }
    }

    fn comment(&self) -> Option<Token> {
        // Comments continue from any '#' character until a line break, or end of the file. Note, that tokenize.py returns an ErrorToken, whose value is the entire comment if the comment contains a \r with no subsequent \n. This doesn't follow that implementation detail.

        let mut characters = self.source[self.position..].char_indices();
        let value = loop {
            let candidate_comment_value = match characters.next() {
                Some((byte_index, '\n')) => 
                    Some(&self.source[self.position..(self.position + byte_index)]),
                Some((byte_index, '\r')) =>
                    match characters.next() {
                        Some((_, '\n')) => 
                            Some(&self.source[self.position..(self.position + byte_index)]),
                        _ => None,
                    },
                Some(_) => None,
                // This is the case that the comment token goes until the end of the source string with no trailing line break.
                None => Some(&self.source[self.position..]),
            };

            if let Some(comment_value) = candidate_comment_value {
                break String::from(comment_value);
            }
        };

        Some(Token {
            token_type: TokenType::Comment,
            value,
        })
    }

    fn newline(&self, value: &'a str) -> Option<Token> {
        let token_type = if self.parentheses_level > 0 {
            TokenType::NewlineContinuation
        } else {
            TokenType::NewlineLogical
        };

        Some(Token {
            token_type,
            value: String::from(value),
        })
    }

    fn name(&self) -> Option<Token> {
        let mut characters = self.source[self.position..].char_indices();
        let value = loop {
            if let Some((byte_index, character)) = characters.next() {
                if !UnicodeXID::is_xid_continue(character) {
                    break &self.source[self.position..(self.position + byte_index)];
                }
            } else {
                // This is the case that the name token goes until the end of the source string with no trailing line break.
                break &self.source[self.position..];
            }
        };

        Some(Token {
            token_type: TokenType::Name,
            // All identifiers are converted to nfkc normalized form. See PEP 3131.
            value: value.nfkc().collect::<String>(),
        })
    }

    fn number(&self) -> Option<Token> {
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
    fn simple_token_three_chars_exact_match() {
        let tokenizer = Tokenizer::new(">>=");
        let expected = Some(Token {
            token_type: TokenType::RightShiftEqual,
            value: String::from(">>="),
        });
        let actual = tokenizer.simple(">>=", TokenType::RightShiftEqual);
        assert_eq!(expected, actual);
    }

    #[test]
    fn simple_token_two_chars_exact_match() {
        let tokenizer = Tokenizer::new(">>");
        let expected = Some(Token {
            token_type: TokenType::RightShift,
            value: String::from(">>"),
        });
        let actual = tokenizer.simple(">>", TokenType::RightShift);
        assert_eq!(expected, actual);
    }

    #[test]
    fn simple_token_one_char_exact_match() {
        let tokenizer = Tokenizer::new(">");
        let expected = Some(Token {
            token_type: TokenType::Greater,
            value: String::from(">"),
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
            value: String::from("&"),
        });
        let actual = tokenizer.next();
        assert_eq!(expected, actual);
    }

    #[test]
    fn ampersand_last_character_in_source() {
        let mut tokenizer = Tokenizer::new("&");
        let expected = Some(Token {
            token_type: TokenType::Ampersand,
            value: String::from("&"),
        });
        let actual = tokenizer.next();
        assert_eq!(expected, actual);
    }

    #[test]
    fn ampersand_equal_with_more_characters() {
        let mut tokenizer = Tokenizer::new("&=abcd");
        let expected = Some(Token {
            token_type: TokenType::AmpersandEqual,
            value: String::from("&="),
        });
        let actual = tokenizer.next();
        assert_eq!(expected, actual);
    }

    #[test]
    fn ampersand_equal_last_character_in_source() {
        let mut tokenizer = Tokenizer::new("&=");
        let expected = Some(Token {
            token_type: TokenType::AmpersandEqual,
            value: String::from("&="),
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
