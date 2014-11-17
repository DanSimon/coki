use regex::{Captures, Regex};
use peruse::parsers::{RegexCapturesParser, RegexLiteralParser, Parser};

use grammar::*;

type Lexer<'a> = Box<Parser<'a, &'a str, Token> + 'a>;


pub fn token<'a>() -> Box<Parser<'a, &'a str, Vec<Token>> + 'a> {

  macro_rules! literal {
    ($reg: expr, $tok: expr) => {
      map!(
        RegexLiteralParser{regex : Regex::new((String::from_str(r"^[ \t]*") + $reg).as_slice()).unwrap()},
        |&: ()| $tok
      )
    }
  }

  //changing these to values creates weird conflicting lifetime errors
  let ident = map!(
    RegexCapturesParser{regex : Regex::new(r"^[ \t]*([a-zA-Z]\w*)[ \t]*").unwrap()},
    |&: caps: Captures<'a>| Ident(from_str(caps.at(1)).unwrap())
  );

  let number = map!(
    RegexCapturesParser{regex : Regex::new(r"^[ \t]*(\d+)[ \t]*").unwrap()},
    |&: caps: Captures<'a>| Number(from_str(caps.at(1)).unwrap())
  );

  box rep!(or!(
    literal!("out",         OutputCmd),
    literal!("if",          IfKeyword),
    literal!("else",        ElseKeyword),
    literal!("while",       WhileKeyword),
    literal!(r"\r?\n\s*",   NewLine),
    literal!(r"\(\s*",      OpenParen),
    literal!(r"\)",         CloseParen),
    literal!(r"\{\s*",      OpenBrace),
    literal!(r"\}",         CloseBrace),
    literal!("==",          Cmp(CEq)),
    literal!("!=",          Cmp(CNeq)),
    literal!(">=",          Cmp(CGeq)),
    literal!("<=",          Cmp(CLeq)),
    literal!(">",           Cmp(CGt)),
    literal!("<",           Cmp(CLt)),
    literal!(r"\+",         PlusSign),
    literal!("-",           MinusSign),
    literal!("=",           Equals),
    literal!(r"\*",         MultSign),
    literal!("/",           DivideSign),
    literal!(r"%",           ModuloSign),
    number,
    ident
  ))

}
