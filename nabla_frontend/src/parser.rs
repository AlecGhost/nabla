use crate::token::{Token, TokenStream};
pub use error::*;

mod error;
#[cfg(test)]
mod tests;
mod utility;

type IResult<'a, T> = nom::IResult<TokenStream<'a>, T, ParserError<'a>>;
