use super::*;
use pretty_assertions::assert_eq;

#[test]
fn empty() {
    let src = "";
    let tokens = lex(src);
    assert_eq!(vec![Token::new(TokenType::Eof, 0..0)], tokens);
}

#[test]
fn all_in_one() {
    let src = "@//abc\n \t\r\ntest 0123.456789'\\n'\"xyz\"true false use def let as :=|*::[]{}";
    let tokens = lex(src);
    assert_eq!(
        vec![
            Token::new(TokenType::Unknown("@".to_string()), 0..1)
                .append_error(Error::new(ErrorMessage::Unknown, 0..1)),
            Token::new(TokenType::Comment("//abc\n".to_string()), 1..7),
            Token::new(TokenType::Whitespace(" \t\r\n".to_string()), 7..11),
            Token::new(TokenType::Ident("test".to_string()), 11..15),
            Token::new(TokenType::Whitespace(" ".to_string()), 15..16),
            Token::new(TokenType::Number("0123.456789".to_string()), 16..27),
            Token::new(TokenType::Char("\\n".to_string()), 27..31),
            Token::new(TokenType::String("xyz".to_string()), 31..36),
            Token::new(TokenType::True, 36..40),
            Token::new(TokenType::Whitespace(" ".to_string()), 40..41),
            Token::new(TokenType::False, 41..46),
            Token::new(TokenType::Whitespace(" ".to_string()), 46..47),
            Token::new(TokenType::Use, 47..50),
            Token::new(TokenType::Whitespace(" ".to_string()), 50..51),
            Token::new(TokenType::Def, 51..54),
            Token::new(TokenType::Whitespace(" ".to_string()), 54..55),
            Token::new(TokenType::Let, 55..58),
            Token::new(TokenType::Whitespace(" ".to_string()), 58..59),
            Token::new(TokenType::As, 59..61),
            Token::new(TokenType::Whitespace(" ".to_string()), 61..62),
            Token::new(TokenType::Colon, 62..63),
            Token::new(TokenType::Eq, 63..64),
            Token::new(TokenType::Pipe, 64..65),
            Token::new(TokenType::Star, 65..66),
            Token::new(TokenType::DoubleColon, 66..68),
            Token::new(TokenType::LBracket, 68..69),
            Token::new(TokenType::RBracket, 69..70),
            Token::new(TokenType::LCurly, 70..71),
            Token::new(TokenType::RCurly, 71..72),
            Token::new(TokenType::Eof, 72..72)
        ],
        tokens
    );
}

#[test]
fn number_missing_decimals() {
    let src = "123.";
    let tokens = lex(src);
    assert_eq!(
        vec![
            Token::new(TokenType::Number("123.".to_string()), 0..4)
                .append_error(Error::new(ErrorMessage::MissingDecimals, 4..4)),
            Token::new(TokenType::Eof, 4..4),
        ],
        tokens
    );
}

#[test]
fn char_missing_single_quote() {
    let src = "'a";
    let tokens = lex(src);
    assert_eq!(
        vec![
            Token::new(TokenType::Char("a".to_string()), 0..2)
                .append_error(Error::new(ErrorMessage::MissingClosingSingleQuote, 2..2)),
            Token::new(TokenType::Eof, 2..2),
        ],
        tokens
    );
}

#[test]
fn char_escape() {
    let src = "'\\''";
    let tokens = lex(src);
    assert_eq!(
        vec![
            Token::new(TokenType::Char("\\'".to_string()), 0..4),
            Token::new(TokenType::Eof, 4..4),
        ],
        tokens
    );

}
