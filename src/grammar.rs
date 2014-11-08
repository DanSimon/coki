
use parsers::*;
use regex::{Captures, Regex};
use std::fmt::Show;

#[deriving(Show)]
#[deriving(Clone)]
#[deriving(PartialEq)]
pub enum AddOp {
  Add,
  Subtract,
}

#[deriving(Show)]
#[deriving(Clone)]
#[deriving(PartialEq)]
pub enum MultOp {
  Multiply,
  Divide,
}

#[deriving(Clone)]
#[deriving(Show)]
#[deriving(PartialEq)]
pub struct AddTerm(pub AddOp, pub Expr);

#[deriving(Clone)]
#[deriving(Show)]
#[deriving(PartialEq)]
pub struct MultTerm(pub MultOp, pub Expr);

#[deriving(Show)]
#[deriving(Clone)]
#[deriving(PartialEq)]
pub enum Expr {
  Variable(String),  
  Num(int),
  AddSub(Vec<AddTerm>), //a + b - c + d becomes [(+ a) (+ b) (- c) (+ d)]
  MultDiv(Vec<MultTerm>), 
}

#[deriving(Show)]
#[deriving(Clone)]
#[deriving(PartialEq)]
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
  MinusSign,
  MultSign,
  DivideSign,
  OutputCmd,
  NewLine,
  OpenParen,
  CloseParen,
}


//coercion fn needed for unboxed closures
//without this all kinds of weird errors show up
fn p<'a, I, O>(l: Box<Parser<'a, I, O> + 'a>) -> Box<Parser<'a, I, O> + 'a> {
  l
}

pub macro_rules! or {
  ($a: expr, $b: expr) => {
    p(box OrParser{
      a: box |&:| $a ,
      b: box |&:| $b ,
    }) 
 };
  ($a: expr, $b: expr $(, $c: expr)* ) => {
    p(box OrParser{
      a: box |&:| $a,
      b: box |&:| or!($b, $($c),*),
    }) 
  };
}
pub macro_rules! seq {
  ($a: expr, $b: expr ) => {
    box DualParser{
      first: $a,
      second: $b,
    } 
 };
  ($a: expr, $b: expr $(, $c: expr)* ) => {
    box DualParser{
      first: $a,
      second: seq!($b, $($c),* ),
    } 
  };
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

pub macro_rules! repsep {
  ($rep: expr, $sep: expr, $min: expr) => {
    box RepSepParser{
      rep: $rep,
      sep: $sep,
      min_reps: $min,
    }
  }
}

pub macro_rules! rep {
  ($rep: expr) => {
    box RepParser{
      parser: $rep,
    }
  }
}


type Lexer<'a> = Box<Parser<'a, &'a str, Token> + 'a>;


pub fn token<'a>() -> Box<Parser<'a, &'a str, Vec<Token>> + 'a> {


  macro_rules! literal {
    ($reg: expr, $tok: expr) => {
      p(map!(
        box RegexLiteralParser{regex : Regex::new($reg).unwrap()},
        |&: ()| $tok
      ))
    }
  }

  //changing these to values creates weird conflicting lifetime errors
  fn ident<'a>() -> Lexer<'a> { map!(
    box RegexCapturesParser{regex : Regex::new(r"^[ \t]*([a-z]+)[ \t]*").unwrap()},
    |&: caps: Captures<'a>| Ident(from_str(caps.at(1)).unwrap())
  )}

  fn number<'a>() -> Lexer<'a> { map!(
    box RegexCapturesParser{regex : Regex::new(r"^[ \t]*(\d+)[ \t]*").unwrap()},
    |&: caps: Captures<'a>| Number(from_str(caps.at(1)).unwrap())
  )}
  

  rep!(or!(
    literal!(r"^[ \t]*out", OutputCmd),
    literal!(r"^[ \t]*\r?\n[ \t]*", NewLine),
    literal!(r"^[ \t]*\(", OpenParen),
    literal!(r"^[ \t]*\)", CloseParen),
    number(),
    literal!(r"^[ \t]*\+", PlusSign),
    literal!(r"^[ \t]*-", MinusSign),
    literal!(r"^[ \t]*=", Equals),
    ident(), 
    literal!(r"^[ \t]*\*", MultSign),
    literal!(r"^[ \t]*/", DivideSign)
  ))

}
    

pub type LParser<'a, T> = Box<Parser<'a, &'a [Token], T> + 'a>;


fn assign<'a>() -> LParser<'a, Statement> {
  box MapParser {
    parser: box DualParser {
      first: box DualParser {
        first: variable(),
        second: literal(Equals),
      },
      second: expr()
    },
    mapper: box |&: ((var, _), expr): ((Expr, Token), Expr)| -> Statement match var{
      Variable(name) => Assign(name, expr),
      _ => panic!("FUCK")
    }
  }
}

fn output<'a>() -> LParser<'a, Statement> {
  map!(
    seq!(literal(OutputCmd), expr()),
    |&: (_, var): (Token, Expr)| Output(var)
  )
}

pub fn statement<'a>() -> LParser<'a, Vec<Statement>> {
  rep!(
    map!(
      seq!(
        or!(output(), assign()),
        literal(NewLine)
      ), 
      |&: (stmt, _): (Statement, Token)| stmt
    )
  )
}


fn variable<'a>() -> LParser<'a, Expr> {
  box MatchParser{
    matcher: box |&: input: &Token| match input {
      &Ident(ref str) => Ok(Variable(str.clone())),
      other => Err(format!("Expected variable, got {}", other))
    }
  }
}

type EParser<'a> = LParser<'a, Expr>;

fn expr<'a>() -> EParser<'a> {

  fn number<'a>() -> EParser<'a> {
    box MatchParser{matcher: box |&: input: &Token| -> Result<Expr, String> match input {
      &Number(num) => Ok(Num(num)),
      other => Err(format!("wrong type, expected number, got {}", other))
    }}
  }

  fn term<'a>() -> EParser<'a> {
    or!(paren_expr(), variable(), number())
  }

  fn mult<'a>() -> EParser<'a> {
    map!(
      seq!(
        term(),
        rep!(seq!(or!(literal(MultSign), literal(DivideSign)), term()))
      ),
      |&: (first, rest): (Expr, Vec<(Token, Expr)>)| {
        let mut f = Vec::new();
        f.push(MultTerm(Multiply, first));
        for &(ref sign, ref value) in rest.iter() {
          let s = match *sign {
            MultSign => Multiply,
            DivideSign => Divide,
            _ => panic!("not allowed")
          };
          f.push(MultTerm(s, value.clone())); //maybe box the value instead
        }
        MultDiv(f)
      }
    )
  }

  fn simple_expr<'a>() -> EParser<'a> {
    or!(mult(), term()) 
  }

  fn plus<'a>() -> EParser<'a> {
    map!(
      seq!(
        simple_expr(),
        rep!(seq!(or!(literal(PlusSign), literal(MinusSign)), simple_expr()))
      ),
      |&: (first, rest): (Expr, Vec<(Token, Expr)>)| {
        let mut f = Vec::new();
        f.push(AddTerm(Add, first));
        for &(ref sign, ref value) in rest.iter() {
          let s = match *sign {
            PlusSign => Add,
            MinusSign => Subtract,
            _ => panic!("not allowed")
          };
          f.push(AddTerm(s, value.clone()));
        }
        AddSub(f)
      }
    )
  }

  fn paren_expr<'a>() -> EParser<'a> {
    box MapParser {
      parser: box DualParser {
        first: box DualParser {
          first: literal(OpenParen),
          second: expr(),
        },
        second: literal(CloseParen),
      },
      mapper: box |&: ((_, e), _): ((Token, Expr), Token)| e
    }
  }

  plus()
}

macro_rules! assert_pat {
  ($actual: expr, $expected: pat) => {
    match $actual{
      $expected => {}
      other => panic!(format!("Assert match failed: got '{}'", other))
    }
  }
}

fn test_parser<'a, I, O: PartialEq + Show>(input: I, parser: &Parser<'a, I, O>, expected: O) {
  match parser.parse(input) {
    Ok((output, rest)) => {
      assert_eq!(output, expected);
    },
    Err(err) => panic!(err)
  }
}

#[test]
fn test_term() {
  let parser = expr();
  let input = [Number(5)];
  let expected = AddSub(vec![AddTerm(Add, Num(5))]);
  test_parser(input.as_slice(), &*parser, expected);
}

#[test]
fn test_plus_sequence() {
  let parser = expr();
  let input = [Number(5), PlusSign, Number(3)];
  let expected = AddSub(vec![AddTerm(Add, Num(5)), AddTerm(Add, Num(3))]);
  test_parser(input.as_slice(), &*parser, expected);
}

#[test]
fn test_simple_assign() {
  let parser = assign();
  let input = [Ident(from_str("x").unwrap()), Equals, Number(7)];
  let expected = Assign(from_str("x").unwrap(), AddSub(vec![AddTerm(Add, Num(7))]));
  test_parser(input.as_slice(), &*parser, expected);
}

#[test]
fn test_simple_output() {
  let parser = output();
  let input = [OutputCmd, Number(4)];
  let expected = Output(AddSub(vec![AddTerm(Add, Num(4))]));
  test_parser(input.as_slice(), &*parser, expected);

}


