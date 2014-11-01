#![feature(globs)]
#![feature(phase)]
#![feature(unboxed_closures)]
//extern crate regex_macros;
//extern crate regex;
//extern crate parsers;

use std::collections::HashMap;
use parsers::*;

pub mod parsers;



fn main() {

  //let parser = CharP
  /*
  let prog = vec![
    Assign("x", Num(3)), 
    Output(Variable("x")),
    Assign("x", Plus(box Variable("x"), box Num(6))), 
    Assign("y", Variable("x")),
    Assign("z", Plus(box Variable("x"), box Variable("y"))),
    Output(Variable("z"))
  ];

  run(&prog);
  */

  let p2 = "x = 3 + 4";
  let t = tokenize(p2);
  for x in t.iter() {
    println!("{}", x);
  }


  /*
  fn simplify<'a, I, O>(b: Box<Parser<'a, I, O> + 'a>) -> ParserGenerator<'a, I, O> {
    box |&:| b
  }
  */

  /*
  let rep_eq   = RepParser{
    reps: 3,
    parser: &OrParser{a: box |&:| box match_num as Box<Parser<&[Token], int>>, b: box |&:| box LiteralParser{literal: Equals} as Box<Parser<&[Token], Token>>}
  };
  */

  fn expr<'a>() -> Box<Parser<'a, &'a [Token], Expr> + 'a> {

    fn match_num<'a>() -> MatchParser<'a Token, Expr> {
      MatchParser{matcher: box |&: input: &Token| -> Result<Expr, String> match *input {
        Number(num) => Ok(Num(num)),
        _ => Err(format!("wrong type"))
      }}
    }

    let plus = |&:| {
      MapParser{
        parser: RepSepParser{
          rep: match_num(),
          sep: literal(PlusSign),
          min_reps : 2
        },
        mapper: box |&: ops: Vec<Expr>| Plus(ops)
      }
    };

    let simple_expr = OrParser{
      b: box |&:| match_num() ,
      a: box plus 
    };

    let mult = MapParser{
      parser: RepSepParser{
        rep: simple_expr,
        sep: literal(MultSign),
        min_reps : 2
      },
      mapper: box |&: ops: Vec<Expr>| Mult(ops)
    };

    box mult
  }

  let e = [Number(11), MultSign, Number(13), PlusSign, Number(14)];

  let parser = expr();
  println!("{}", parser.parse(&e));




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
        "out" => {build.push(OutputCmd)}
        "" => {}
        other => { build.push(Ident(String::from_str(other)))}
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
  let mut env: HashMap<&str, int> = HashMap::new();
  for s in prog.iter() {
    match *s {
      Assign(var, ref expr) => {
        match eval(expr, &env) {
          Ok(res)   => {env.insert(var, res);}
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

fn eval(expr: &Expr, env: &HashMap<&str, int>) -> Result<int, String> {
  match *expr {
    Variable(var) => match env.find(&var) {
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
            total += value;
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
  Variable(&'static str),  
  Num(int),
  Plus(Vec<Expr>), //a + b + c + d
  Mult(Vec<Expr>)
}

enum Statement {
  Assign(&'static str, Expr),
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
}
