#![feature(globs)]
#![feature(phase)]
#![feature(unboxed_closures)]
#![feature(macro_rules)]
extern crate regex;
//extern crate regex_macros;
//extern crate parsers;

use std::collections::HashMap;
use parsers::*;
use grammar::*;
use regex::*;

pub mod parsers;
pub mod grammar;





fn main() {


  
  let p2 = "x = ( 3 + 4 ) * 3
  y = 6 + x
  out x + 7 * y
  ";

  let lexer = token();
  println!("{}", lexer.parse(p2));
  
  let tokens = tokenize(p2);
  match lexer.parse(p2) {
    Ok((mut tokens, rest)) => {
      if rest != "" {
        println!("Parser error at: {}", rest)
      } else {
        //tokens.push(NewLine);//so user doesn't have to have a terminating \n
        let parser = statement();
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
    },
    Err(err) => {
      println!("Lexer error: {}", err);
    }
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
  

