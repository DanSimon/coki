#![feature(globs)]
#![feature(phase)]
#![feature(unboxed_closures)]
#![feature(macro_rules)]
//extern crate regex_macros;
//extern crate regex;
//extern crate parsers;

use std::collections::HashMap;
use parsers::*;

pub mod parsers;



fn main() {

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

  fn statement<'a>() -> Box<Parser<'a, &'a [Token], Vec<Statement>> + 'a> {
    box RepParser{
      parser: box MapParser{
        parser: box DualParser {
          first: box OrParser {
            b: box |&:| assign(),
            a: box |&:| output(),
          },
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

  //let e = [Number(11), MultSign, Number(13), MultSign, Number(17), PlusSign, Number(14)];
  
  let p2 = "x = ( 3 + 4 ) * 3 \n out x + 7 * x";
  
  let parser = statement();
  let tokens = tokenize(p2);
  match parser.parse(tokens.as_slice()) {
    Ok((exp, rest)) => {
      if rest.len() > 0 {
        println!("Error: unexpected token {}", rest[0]);
      } else {
        run(&exp);
      }
    }
    Err(err) => {println!("Parse Error: {}", err);}
  };


}

fn tokenize<'a>(input: &'a str) -> Vec<Token> {
  let mut lines = input.split('\n');
  let mut build: Vec<Token> = Vec::new();
  for line in lines {
    let mut toks = line.split(' ');
    for tok in toks {
      match tok.trim() {
        "=" => {build.push(Equals);}
        "+" => {build.push(PlusSign);}
        "*" => {build.push(MultSign);}
        "(" => {build.push(OpenParen);}
        ")" => {build.push(CloseParen);}
        "out" => {build.push(OutputCmd)}
        "" => {}
        other => {
          match from_str(other) {
            Some(num) => {
              build.push(Number(num));
            }
            None => {
              build.push(Ident(String::from_str(other)));
            }
          }
        }
      };
    }
    build.push(NewLine);
  }
  build
}

/*
trait TokenParser<T> {
  fn parse<'a>(tokens: &'a [Token]>)
*/

fn run(prog: &Vec<Statement>) {
  let mut env: HashMap<String, int> = HashMap::new();
  for s in prog.iter() {
    match *s {
      Assign(ref var, ref expr) => {
        match eval(expr, &env) {
          Ok(res)   => {env.insert(var.clone(), res);}
          Err(err)  => {
            println!("ERROR: {}", err);
            return;
          }
        }
      }
      Output(ref expr) => match eval(expr, &env) {
        Ok(val) => {println!("{}", val)}
        Err(err) => {
          println!("ERROR: {}", err)
          return;
        }
      }
    }
  }
}

fn eval(expr: &Expr, env: &HashMap<String, int>) -> Result<int, String> {
  match *expr {
    Variable(ref var) => match env.find(var) {
      Some(val) => Ok(*val),
      None => Err(format!("Undefined var {}", var)),
    },
    Num(val) => Ok(val),
    Plus(ref ops) => {
      let mut sum = 0i;
      for op in ops.iter() {
        match eval(op, env) {
          Ok(value) => {
            sum += value;
          }
          Err(err) => {
            return Err(err);
          }
        }
      }
      Ok(sum)
    },
    Mult(ref ops) => {
      let mut total = 1i;
      for op in ops.iter() {
        match eval(op, env) {
          Ok(value) => {
            total *= value;
          }
          Err(err) => {
            return Err(err);
          }
        }
      }
      Ok(total)
    }
  }
}
  

#[deriving(Show)]
enum Expr {
  Variable(String),  
  Num(int),
  Plus(Vec<Expr>), //a + b + c + d
  Mult(Vec<Expr>)
}

enum Statement {
  Assign(String, Expr),
  Output(Expr),
}

#[deriving(Show)]
#[deriving(Eq)]
#[deriving(PartialEq)]
#[deriving(Clone)]
enum Token {
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
