
use parsers::*;
use regex::{Captures, Regex};

pub enum AddOp {
  Add,
  Subtract,
}

pub enum MultOp {
  Multiply,
  Divide,
}

type AddTerm = (AddOp, Expr);
type MultTerm = (MultOp, Expr);

#[deriving(Show)]
pub enum Expr {
  Variable(String),  
  Num(int),
  AddSub(Vec<AddTerm>), //a + b - c + d becomes [(+ a) (+ b) (- c) (+ d)]
  MultDiv(Vec<MultTerm>), 
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
  

  box RepParser{
    parser: or!(
      literal!(r"^[ \t]*out", OutputCmd),
      literal!(r"^[ \t]*\r?\n[ \t]*", NewLine),
      literal!(r"^[ \t]*\(", OpenParen),
      literal!(r"^[ \t]*\)", CloseParen),
      number(),
      literal!(r"^[ \t]*\+", PlusSign),
      literal!(r"^[ \t]*=", Equals),
      ident(), 
      literal!(r"^[ \t]*\*", MultSign)
    )
  }

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
  box MapParser {
    parser : box DualParser {
      first: literal(OutputCmd),
      second: expr(),
    },
    mapper: box |&: (_, var): (Token, Expr)| Output(var)
  }
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

  fn match_num<'a>() -> EParser<'a> {
    box MatchParser{matcher: box |&: input: &Token| -> Result<Expr, String> match input {
      &Number(num) => Ok(Num(num)),
      other => Err(format!("wrong type, expected number, got {}", other))
    }}
  }

  fn term<'a>() -> EParser<'a> {
    box OrParser{
      a: box |&:| paren_expr(),
      b: box |&:| box OrParser {
        a: box |&:| variable(),
        b: box |&:| match_num(),
      } as EParser<'a>
    }
  }

  fn mult<'a>() -> EParser<'a> {
    box MapParser{
      parser: box RepSepParser{
        rep: term(),
        sep: literal(MultSign),
        min_reps : 2
      },
      mapper: box |&: ops: Vec<Expr>| Mult(ops)
    }
  }

  fn simple_expr<'a>() -> EParser<'a> {
    box OrParser{
      b: box |&:| term(), 
      a: box |&:| mult() 
    }
  }

  fn plus<'a>() -> EParser<'a> {
    box MapParser{
      parser: box RepSepParser{
        rep: simple_expr(),
        sep: literal(PlusSign),
        min_reps : 2
      },
      mapper: box |&: ops: Vec<Expr>| Plus(ops)
    }
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

  let expr = box OrParser{
      a: box |&:| plus(),
      b: box |&:| simple_expr(),
  };

  expr
}
