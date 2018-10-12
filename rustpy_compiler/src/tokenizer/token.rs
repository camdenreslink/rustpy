#[derive(PartialEq, Debug)]
pub enum TokenType {
    // Temporary comments next to token type indicates that the token has been matched in tokenizer
    Ampersand,       //
    AmpersandEqual,  //
    At,              //
    AtEqual,         //
    Circumflex,      //
    CircumflexEqual, //
    Colon,           //
    Comma,           //
    Comment,         //
    Dedent,
    Dot,              //
    DoubleEqual,      //
    DoubleSlash,      //
    DoubleSlashEqual, //
    DoubleStar,       //
    DoubleStarEqual,  //
    Ellipsis,         //
    Encoding,
    EndMarker,
    Equal, //
    ErrorToken,
    Greater,      //
    GreaterEqual, //
    Indent,
    LeftSquareBracket, //
    LeftBrace,         //
    LeftParenthesis,   //
    LeftShift,         //
    LeftShiftEqual,    //
    Less,              //
    LessEqual,         //
    MinusEqual,        //
    Minus,             //
    Name,              //
    NewlineContinuation,
    NewlineLogical,
    NotEqual, //
    Number,   //
    Operator,
    Percent,            //
    PercentEqual,       //
    Plus,               //
    PlusEqual,          //
    RightArrow,         //
    RightBrace,         //
    RightParenthesis,   //
    RightSquareBracket, //
    RightShift,         //
    RightShiftEqual,    //
    Semicolon,          //
    Slash,              //
    SlashEqual,         //
    Star,               //
    StarEqual,          //
    String,
    Tilde,            //
    VerticalBar,      //
    VerticalBarEqual, //
}

#[derive(PartialEq, Debug)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub value: &'a str,
}
