use super::Span;
use crate::token::{Error, ErrorMessage};
use nom::{bytes::complete::take_while1, character::is_alphanumeric};

type IResult<'a, O> = nom::IResult<Span<'a>, O>;

/// Parser for alphanumeric characters or underscores.
/// At least one character must be recognized.
pub(super) fn alpha_numeric1(input: Span) -> IResult<Span> {
    take_while1(is_alpha_numeric)(input)
}

/// Checks if provided char is alphanumeric or an underscore.
pub(super) fn is_alpha_numeric(c: char) -> bool {
    is_alphanumeric(c as u8) || c == '_'
}

/// Tries to parse the input with the given parser.
/// If parsing succeeds, the result of inner is returned.
/// If parsing fails, an error with the provided message is reported.
pub(super) fn expect<'a, O, F>(
    mut f: F,
    error_msg: ErrorMessage,
) -> impl FnMut(Span<'a>) -> IResult<Result<O, Error>>
where
    F: FnMut(Span<'a>) -> IResult<'a, O>,
{
    move |input: Span| {
        let error_pos = input.location_offset();
        match f(input.clone()) {
            Ok((input, out)) => Ok((input, Ok(out))),
            Err(_) => {
                let err = Error::new(error_msg.clone(), error_pos..error_pos);
                Ok((input, Err(err)))
            }
        }
    }
}
