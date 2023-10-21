use crate::{
    ast::AstInfo,
    parser::{self, IResult, ParserError, ParserErrorKind},
    token::TokenStream,
};
use nom::bytes::complete::take;

pub(super) fn expect<'a, O, F>(
    mut expected: F,
    error_message: parser::ErrorMessage,
) -> impl FnMut(TokenStream<'a>) -> IResult<'a, Option<O>>
where
    F: FnMut(TokenStream<'a>) -> IResult<'a, O>,
{
    move |input| match expected(input) {
        Ok((input, o)) => Ok((input, Some(o))),
        Err(nom::Err::Error(err)) => {
            let mut input = err.input;
            let pos = input.location_offset();
            let error_pos = if pos > 0 { pos - 1 } else { 0 };
            let error = parser::Error::new(error_message.clone(), error_pos..error_pos);
            input.append_error(error);
            Ok((input, None))
        }
        Err(_) => panic!("expect: unexpected error"),
    }
}

pub(super) fn ignore_until<'a, O, F>(mut pattern: F) -> impl FnMut(TokenStream<'a>) -> IResult<()>
where
    F: FnMut(TokenStream<'a>) -> IResult<'a, O>,
{
    move |input| match pattern(input) {
        Ok((input, _)) => Err(nom::Err::Error(ParserError {
            input,
            kind: ParserErrorKind::IgnoreUntil,
        })),
        Err(nom::Err::Error(err)) => {
            let mut input = err.input;
            loop {
                match pattern(input) {
                    Ok((input, _)) => return Ok((input, ())),
                    Err(nom::Err::Error(err)) => {
                        match take::<usize, TokenStream<'a>, ParserError<'a>>(1)(err.input) {
                            Ok((i, _)) => input = i,
                            Err(nom::Err::Error(err)) => return Err(nom::Err::Error(err)),
                            Err(_) => panic!("ignore_until: unexpected error"),
                        }
                    }
                    Err(_) => panic!("ignore_until: unexpected error"),
                }
            }
        }
        Err(_) => panic!("ignore_until: unexpected error"),
    }
}

pub(super) fn info<'a, O, F>(
    mut parser: F,
) -> impl FnMut(TokenStream<'a>) -> IResult<'a, (O, AstInfo)>
where
    F: FnMut(TokenStream<'a>) -> IResult<'a, O>,
{
    move |input| {
        let start = input.location_offset();
        let error_len = input.error_buffer.len();
        match parser(input) {
            Ok((input, o)) => {
                let end = input.location_offset();
                let range = start..end;
                let info = AstInfo::new(range);
                Ok((input, (o, info)))
            }
            Err(nom::Err::Error(mut err)) => {
                // remove errors that where added in the meantime
                err.input.error_buffer.truncate(error_len);
                Err(nom::Err::Error(err))
            }
            Err(_) => panic!("info: unexpected error"),
        }
    }
}
