//! Summary
//! -------
//!
//! Control is routed to the functions in this module when the token type has been unambiguously determined. The token begins at the start of the string slice provided.

use super::token::{Token, TokenType};

/// A comment starts with a hash character (#) that is not part of a string literal, and ends at the end of the physical line.
pub fn comment<'a>(source: &'a str) -> Token<'a> {
    Token {
        token_type: TokenType::Comment,
        value: "",
    }
}

pub fn name<'a>(source: &'a str) -> Token<'a> {
    Token {
        token_type: TokenType::Name,
        value: "",
    }
}

pub fn number<'a>(source: &'a str) -> Token<'a> {
    Token {
        token_type: TokenType::Number,
        value: "",
    }
}

/// A fraction is a numeric literal that starts with a '.' character. For example: ".50053"
pub fn fraction<'a>(source: &'a str) -> Token<'a> {
    let mut token_end_position = source.len();

    let characters = source.char_indices().peekable();

    // We know that the first character must be a '.', so we can skip it.
    for (byte_position, character) in characters.skip(1) {
        let valid_char = match character {
            '0'..='9' | '_' => true,
            _ => false,
        };
        if !valid_char {
            token_end_position = byte_position;
            break;
        };
    }

    Token {
        token_type: TokenType::Number,
        value: &source[0..token_end_position],
    }
}

mod helpers {}

#[cfg(test)]
mod tests {
    use super::*;

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
}
