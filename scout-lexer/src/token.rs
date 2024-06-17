#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TokenKind {
    Illegal,
    EOF,
    Ident,
    Int,
    Float,
    Str,

    Comma,
    Colon,
    Pipe,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Select,
    SelectAll,
    Assign,
    EQ,
    NEQ,
    Plus,
    Minus,
    Asterisk,
    Slash,

    // Keywords
    If,
    For,
    In,
    Do,
    End,
    Goto,
    Scrape,
    Screenshot,
    True,
    False,
    Def,
    Null,
    Return,
    Use,
}

impl TokenKind {
    pub fn is_to_keyword(literal: &str) -> Option<Self> {
        use TokenKind::*;
        match literal {
            "for" => Some(For),
            "in" => Some(In),
            "if" => Some(If),
            "do" => Some(Do),
            "end" => Some(End),
            "goto" => Some(Goto),
            "scrape" => Some(Scrape),
            "screenshot" => Some(Screenshot),
            "true" => Some(True),
            "false" => Some(False),
            "def" => Some(Def),
            "null" => Some(Null),
            "return" => Some(Return),
            "use" => Some(Use),
            _ => None,
        }
    }

    pub fn is_infix(&self) -> bool {
        use TokenKind::*;
        match self {
            EQ => true,
            NEQ => true,
            Plus => true,
            Minus => true,
            Asterisk => true,
            Slash => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: String,
}

impl Token {
    pub fn new(kind: TokenKind, literal: String) -> Self {
        Self { kind, literal }
    }
}
