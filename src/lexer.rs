use regex::{Captures, Regex};
use peruse::slice_parsers::*;
use peruse::string_parsers::*;

use grammar::*;
use std::str::FromStr;

//type Lexer = SliceParser<I=str, O=Token>;


pub fn token() -> Box<SliceParser<I=str, O=Vec<Token>>> {

  let lt = |s: &str, t: Token| {
    str_lit((String::from_str(r"^[ \t]*") + s).as_str(), t)
  };

  Box::new(lt("foo", Token::NewLine).repeat())

  //changing these to values creates weird conflicting lifetime errors

  /*
  let ident = map!(
    RegexCapturesParser{regex : Regex::new(r"^[ \t]*([a-zA-Z]\w*)[ \t]*").unwrap()},
    |&: caps: Captures<'a>| Token::Ident(String::from_str(caps.at(1).unwrap()))
  );

  let number = map!(
    RegexCapturesParser{regex : Regex::new(r"^[ \t]*(\d+)[ \t]*").unwrap()},
    |&: caps: Captures<'a>| Token::Number(FromStr::from_str(caps.at(1).unwrap()).unwrap())
  );
  */

  /*

  Box::new( rep!(or!(
    literal!("out",         Token::OutputCmd),
    literal!("if",          Token::IfKeyword),
    literal!("else",        Token::ElseKeyword),
    literal!("while",       Token::WhileKeyword),
    literal!(r"\r?\n\s*",   Token::NewLine),
    literal!(r"\(\s*",      Token::OpenParen),
    literal!(r"\)",         Token::CloseParen),
    literal!(r"\{\s*",      Token::OpenBrace),
    literal!(r"\}",         Token::CloseBrace),
    literal!("==",          Token::Cmp(Comparator::CEq)),
    literal!("!=",          Token::Cmp(Comparator::CNeq)),
    literal!(">=",          Token::Cmp(Comparator::CGeq)),
    literal!("<=",          Token::Cmp(Comparator::CLeq)),
    literal!(">",           Token::Cmp(Comparator::CGt)),
    literal!("<",           Token::Cmp(Comparator::CLt)),
    literal!(r"\+",         Token::PlusSign),
    literal!("-",           Token::MinusSign),
    literal!("=",           Token::Equals),
    literal!(r"\*",         Token::MultSign),
    literal!("/",           Token::DivideSign),
    literal!(r"%",           Token::ModuloSign),
    number,
    ident
  )))
  */

}
