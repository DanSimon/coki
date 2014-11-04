#![feature(macro_rules)]

use parsers::*;
use regex::{Captures, Regex};

#[deriving(Show)]
pub enum Expr {
  Variable(String),  
  Num(int),
  Plus(Vec<Expr>), //a + b + c + d
  Mult(Vec<Expr>)
}

pub enum Statement {
  Assign(String, Expr),
  Output(Expr),
}

#[deriving(Show)]
#[deriving(Eq)]
#[deriving(PartialEq)]
#[deriving(Clone)]
pub enum Token {
  Equals,
  Ident(String),
  Number(int),
  PlusSign,
  MultSign,
  OutputCmd,
  NewLine,
  OpenParen,
  CloseParen,
}

pub macro_rules! or {
  ($a: expr, $b: expr) => {
    box OrParser{
      a: box |&:| $a,
      b: box |&:| $b,
    }
  };
  ($a: expr, $b: expr, $c: expr) => {
    box OrParser{
      a: box |&:| $a,
      b: box |&:| or!($b, $c),
    }
  }
}

pub macro_rules! map {
  ($a: expr, $b: expr) => {
    box MapParser{
      parser: $a,
      mapper: box $b
    }
  }
}

pub macro_rules! link {
  ($a: expr, $b: expr) => {
    box DualParser{
      first: $a,
      second: $b,
    }
  }
}

type Lexer<'a> = Box<Parser<'a, &'a str, Token> + 'a>;

pub fn token<'a>() -> Lexer<'a> {

  macro_rules! literal{
    ($reg: expr, $lit: expr ) => {map!(
      box RegexLiteralParser{regex : Regex::new($reg).unwrap()},
      |&: ()| $lit
    ) as Lexer<'a>}
  }

  macro_rules! lor{
    ($a: expr, $b: expr) => (or!($a,$b) as Lexer<'a>)
  }

  fn ident<'a>() -> Lexer<'a> { map!(
    box RegexCapturesParser{regex : Regex::new(r"^[ \t]*([a-z]+)[ \t]*").unwrap()},
    |&: caps: Captures<'a>| Ident(from_str(caps.at(1)).unwrap())
  )}

  fn number<'a>() -> Lexer<'a> { map!(
    box RegexCapturesParser{regex : Regex::new(r"^[ \t]*(\d+)[ \t]*").unwrap()},
    |&: caps: Captures<'a>| Number(from_str(caps.at(1)).unwrap())
  )}

  box RepParser{
    parser: lor!(
      literal!(r"^[ \t]*out", OutputCmd),
      lor!(
      literal!(r"^[ \t]*\r?\n[ \t]*", NewLine),
      lor!(
      literal!(r"^[ \t]*\(", OpenParen),
      lor!(
      literal!(r"^[ \t]*\)", CloseParen),
      lor!(
      number(),
      lor!(
      literal!(r"^[ \t]*\+", PlusSign),
      lor!(
      literal!(r"^[ \t]*=", Equals),
      lor!(
      ident(), 
      literal!(r"^[ \t]*\*", MultSign)
      ))))))))
  }

}
    

type LParser<'a> = Box<Parser<'a, &'a [Token], Expr> + 'a>;


fn assign<'a>() -> Box<Parser<'a, &'a [Token], Statement> + 'a> {
  box MapParser {
    parser: box DualParser {
      first: box DualParser {
        first: variable(),
        second: literal(Equals),
      },
      second: expr()
    },
    mapper: box |&: ((var, eq), expr): ((Expr, Token), Expr)| -> Statement match var{
      Variable(name) => Assign(name, expr),
      _ => panic!("FUCK")
    }
  }
}

fn output<'a>() -> Box<Parser<'a, &'a [Token], Statement> + 'a> {
  box MapParser {
    parser : box DualParser {
      first: literal(OutputCmd),
      second: expr(),
    },
    mapper: box |&: (out, var): (Token, Expr)| Output(var)
  }
}

pub fn statement<'a>() -> Box<Parser<'a, &'a [Token], Vec<Statement>> + 'a> {
  box RepParser{
    parser: box MapParser{
      parser: box DualParser {
        first: or!(output(), assign()),
        second: literal(NewLine),
      },
      mapper: box |&: (stmt, nl): (Statement, Token)| stmt
    }
  }
}


fn variable<'a>() -> LParser<'a> {
  box MatchParser{
    matcher: box |&: input: &Token| match input {
      &Ident(ref str) => Ok(Variable(str.clone())),
      other => Err(format!("Expected variable, got {}", other))
    }
  }
}

fn expr<'a>() -> LParser<'a> {

  fn match_num<'a>() -> Box<Parser<'a, &'a[Token], Expr> + 'a> {
    box MatchParser{matcher: box |&: input: &Token| -> Result<Expr, String> match input {
      &Number(num) => Ok(Num(num)),
      other => Err(format!("wrong type, expected number, got {}", other))
    }}
  }

  fn term<'a>() -> LParser<'a> {
    box OrParser{
      a: box |&:| paren_expr(),
      b: box |&:| box OrParser {
        a: box |&:| variable(),
        b: box |&:| match_num(),
      } as LParser<'a>
    }
  }

  fn mult<'a>() -> LParser<'a> {
    box MapParser{
      parser: box RepSepParser{
        rep: term(),
        sep: literal(MultSign),
        min_reps : 2
      },
      mapper: box |&: ops: Vec<Expr>| Mult(ops)
    }
  }

  fn simple_expr<'a>() -> LParser<'a> {
    box OrParser{
      b: box |&:| term(), 
      a: box |&:| mult() 
    }
  }

  fn plus<'a>() -> LParser<'a> {
    box MapParser{
      parser: box RepSepParser{
        rep: simple_expr(),
        sep: literal(PlusSign),
        min_reps : 2
      },
      mapper: box |&: ops: Vec<Expr>| Plus(ops)
    }
  }

  fn paren_expr<'a>() -> LParser<'a> {
    box MapParser {
      parser: box DualParser {
        first: box DualParser {
          first: literal(OpenParen),
          second: expr(),
        },
        second: literal(CloseParen),
      },
      mapper: box |&: ((o, e), c): ((Token, Expr), Token)| e
    }
  }

  let expr = box OrParser{
      a: box |&:| plus(),
      b: box |&:| simple_expr(),
  };

  expr
}
