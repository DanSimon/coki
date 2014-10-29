#![feature(globs)]
//extern crate regex_macros;
//extern crate regex;
//extern crate parsers;

use std::collections::HashMap;
use parsers::*;

pub mod parsers;



fn main() {

  //let parser = CharP

  let prog = vec![
    Assign("x", Num(3)), 
    Output(Variable("x")),
    Assign("x", Plus(box Variable("x"), box Num(6))), 
    Assign("y", Variable("x")),
    Assign("z", Plus(box Variable("x"), box Variable("y"))),
    Output(Variable("z"))
  ];

  run(&prog);

  let p2 = "x = 3 + 4";
  let t = tokenize(p2);
  for x in t.iter() {
    println!("{}", x);
  }

  let eq    = LiteralParser{literal: Equals};
  let eq2   = LiteralParser{literal: Number(34)};
  let por   = OrParser{a: &eq, b: &eq2};
  let repEq = RepParser{reps: 3, parser: &por};


  let data = [Equals, Equals, Number(34), PlusSign];
  match repEq.parse(data) {
    Ok(k) => {println!("ok");}
    Err(err) => {println!("error: {}", err);}
  }


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
    Plus(box ref a, box ref b) => match eval(a, env) {
      Ok(va) => match eval(b, env) {
        Ok(vb) => Ok(va + vb),
        Err(err) => Err(err),
      },
      Err(err) => Err(err),
    }
  }
}
  

enum Expr {
  Variable(&'static str),  
  Num(int),
  Plus(Box<Expr>, Box<Expr>),
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
  OutputCmd,
  NewLine,
}
