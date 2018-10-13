#[derive(PartialEq, Debug, Clone, Copy)]
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
    Dedent, //
    Dot,
    DoubleEqual,
    DoubleSlash,
    DoubleSlashEqual,
    DoubleStar,
    DoubleStarEqual,
    Ellipsis,
    Encoding, // Not sure if this token is actually required. All internal processing is in UTF-8
    EndMarker, // Not sure if this token is actually required
    Equal,
    ErrorToken, //
    Greater,
    GreaterEqual,
    Indent, //
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
    Number,   //
    Operator, // This token appears to only be meaningfully used by tokenize.py, not tokenizer.c
    Percent,
    PercentEqual,
    Plus,
    PlusEqual,
    RightArrow,
    RightBrace,
    RightParenthesis,
    RightSquareBracket,
    RightShift,
    RightShiftEqual,
    Semicolon,
    Slash,
    SlashEqual,
    Star,
    StarEqual,
    String, //
    Tilde,
    VerticalBar,
    VerticalBarEqual,
}

#[derive(PartialEq, Debug)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub value: &'a str,
}
