#![feature(globs)]
#![feature(phase)]
#![feature(unboxed_closures)]
#![feature(macro_rules)]
extern crate regex;
#[phase(plugin)] extern crate peruse;
extern crate peruse;

use std::collections::HashMap;
use peruse::parsers::*;
use grammar::*;
use parser::block;
use lexer::token;
use std::os;
use std::io::File;


pub mod lexer;
pub mod grammar;
pub mod parser;

fn main() {
  
  let args = os::args();

  let file = &args[1];
  
  let contents = File::open(&Path::new(file.as_slice())).read_to_string();

  match contents {
    Ok(raw) => {
      let lexer = token();
      match lexer.parse(raw.as_slice()) {
        Ok((tokens, rest)) => {
          if rest != "" {
            println!("Parser error at: {}", rest)
          } else {
            let parser = block();
            match parser.parse(tokens.as_slice()) {
              Ok((Block(stmts), rest)) => {
                if rest.len() > 0 {
                  println!("Error: unexpected token {}", rest[0]);
                } else {
                  run(&stmts);
                }
              }
              Err(err) => {println!("Parse Error: {}", err);}
            };
          }
        },
        Err(err) => {
          println!("Lexer error: {}", err);
        }
      }
    }
    Err(err) => {println!("Error Reading File: {}", err);}
  }

}

type Environment = HashMap<String, int>;


fn run(prog: &Vec<Statement>) {
  fn run_internal(prog: &Vec<Statement>, env: &mut Environment) {
    for s in prog.iter() {
      match *s {
        Assign(ref var, ref expr) => {
          match eval(expr, env) {
            Ok(res)   => {env.insert(var.clone(), res);}
            Err(err)  => {
              println!("ERROR: {}", err);
              return;
            }
          }
        },
        Output(ref expr) => match eval(expr, env) {
          Ok(val) => {println!("{}", val)}
          Err(err) => {
            println!("ERROR: {}", err)
            return;
          }
        },
        If(ref lhs, ref cmp, ref rhs, ref then_block, ref else_block) => {
          let is_true = compare(lhs, cmp, rhs, env);
          let &Block(ref p) = if (is_true) { then_block } else { else_block };
          //fixme: need to respect block scopes
          run_internal(p, env);
        }
        While(ref lhs, ref cmp, ref rhs, ref block) => {
          let &Block(ref stmts) = block;
          while(compare(lhs, cmp, rhs, env)) {
            run_internal(stmts, env);
          }
        }
      }
    }
  }
  let mut env: HashMap<String, int> = HashMap::new();
  run_internal(prog, &mut env);
}

fn compare(lhs: &Expr, cmp: &Comparator, rhs: &Expr, env: &Environment) -> bool {
  let l = eval(lhs, env);
  let r = eval(rhs, env);
  match *cmp {
    CEq   => l == r,
    CNeq  => l != r,
    CLt   => l < r,
    CGt   => l > r,
    CLeq  => l <= r,
    CGeq  => l >= r,
  }
}

fn eval(expr: &Expr, env: &HashMap<String, int>) -> Result<int, String> {
  match *expr {
    Variable(ref var) => match env.find(var) {
      Some(val) => Ok(*val),
      None => Err(format!("Undefined var {}", var)),
    },
    Num(val) => Ok(val),
    AddSub(ref ops) => {
      let mut sum = 0i;
      for &AddTerm(ref sign, ref op) in ops.iter() {
        match eval(op, env) {
          Ok(value) => match *sign {
            Add => {sum += value;}
            Subtract => {sum -= value;}
          },
          Err(err) => {
            return Err(err);
          }
        }
      }
      Ok(sum)
    },
    MultDiv(ref ops) => {
      let mut total = 1i;
      for &MultTerm(ref sign,ref op) in ops.iter() {
        match eval(op, env) {
          Ok(value) => match *sign{
            Multiply => {total *= value;}
            Divide  => {total /= value;}
          },
          Err(err) => {
            return Err(err);
          }
        }
      }
      Ok(total)
    }
  }
}
  

