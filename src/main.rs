#![feature(unboxed_closures)]
#![feature(collections)]
#![feature(convert)]
extern crate regex;
extern crate peruse;
//extern crate test;

//#[test]
//use test::Bencher;

use std::collections::HashMap;
use grammar::*;
use parser::program;
use lexer::token;
use std::os;
use std::fs::File;
use std::env::args;
use std::path::Path;
use std::io::Read;


pub mod lexer;
pub mod grammar;
pub mod parser;

fn main() {
  
  let mut args = args();

  let file = args.nth(1).unwrap();
  
  let mut contents = String::new();
  let mut f = File::open(file.as_str()).unwrap();
  f.read_to_string(&mut contents);

  interp(contents.as_str());

}

type Environment = HashMap<String,i32>;

fn interp<'a>(raw: &'a str) {
  let lexer = token();
  match lexer.parse(raw) {
    Ok((tokens, rest)) => {
      if rest != "" {
        println!("Parser error at: {:?}", rest)
      } else {
        let parser = program();
        match parser.parse(tokens.as_slice()) {
          Ok((Block(stmts), rest)) => {
            if rest.len() > 0 {
              println!("Error: unexpected token {:?}", rest[0]);
            } else {
              run(&stmts);
            }
          }
          Err(err) => {println!("Parse Error: {:?}", err);}
        };
      }
    },
    Err(err) => {
      println!("Lexer error: {:?}", err);
    }
  }

}


fn run(prog: &Vec<Statement>) {
  fn runi32ernal(prog: &Vec<Statement>, env: &mut Environment) {
    for s in prog.iter() {
      match *s {
        Statement::Assign(ref var, ref expr) => {
          match eval(expr, env) {
            Ok(res)   => {env.insert(var.clone(), res);}
            Err(err)  => {
              println!("ERROR: {}", err);
              return;
            }
          }
        },
        Statement::Output(ref expr) => match eval(expr, env) {
          Ok(val) => {println!("{}", val)}
          Err(err) => {
            println!("ERROR: {}", err);
            return;
          }
        },
        Statement::If(ref lhs, ref cmp, ref rhs, ref then_block, ref else_block) => {
          let is_true = compare(lhs, cmp, rhs, env);
          if is_true {            
            let &Block(ref p) = then_block;
            runi32ernal(p, env);
          } else {
            match else_block {
              &Some(Block(ref p)) => {runi32ernal(p, env);}
              &None => {}
            }
          }
        }
        Statement::While(ref lhs, ref cmp, ref rhs, ref block) => {
          let &Block(ref stmts) = block;
          while compare(lhs, cmp, rhs, env) {
            runi32ernal(stmts, env);
          }
        }
      }
    }
  }
  let mut env: HashMap<String,i32> = HashMap::new();
  runi32ernal(prog, &mut env);
}

fn compare(lhs: &Expr, cmp: &Comparator, rhs: &Expr, env: &Environment) -> bool {
  let l = eval(lhs, env);
  let r = eval(rhs, env);
  match *cmp {
    Comparator::CEq   => l == r,
    Comparator::CNeq  => l != r,
    Comparator::CLt   => l < r,
    Comparator::CGt   => l > r,
    Comparator::CLeq  => l <= r,
    Comparator::CGeq  => l >= r,
  }
}

fn eval(expr: &Expr, env: &HashMap<String,i32>) -> Result<i32, String> {
  match *expr {
    Expr::Variable(ref var) => match env.get(var) {
      Some(val) => Ok(*val),
      None => Err(format!("Undefined var {}", var)),
    },
    Expr::Num(val) => Ok(val),
    Expr::AddSub(ref ops) => {
      let mut sum = 0;
      for &AddTerm(ref sign, ref op) in ops.iter() {
        match eval(op, env) {
          Ok(value) => match *sign {
            AddOp::Add => {sum += value;}
            AddOp::Subtract => {sum -= value;}
          },
          Err(err) => {
            return Err(err);
          }
        }
      }
      Ok(sum)
    },
    Expr::MultDiv(ref ops) => {
      let mut total = 1;
      for &MultTerm(ref sign,ref op) in ops.iter() {
        match eval(op, env) {
          Ok(value) => match *sign{
            MultOp::Multiply  => {total *= value;}
            MultOp::Divide    => {total /= value;}
            MultOp::Modulo    => {total %= value;} 
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

/*
#[bench]
fn bench_run(b: &mut Bencher) {
    let prog = "n1 = 1
n2 = 1
n = n1 + n2
i = 0

while i < 30 {
  n2 = n1
  n1 = n
  n = n1 + n2
  i = i + 1
}
";
  let lexer = token();
  let (tokens, rest) = lexer.parse(prog).unwrap();
  let parser = program();
  b.iter(|| {
    parser.parse(tokens.as_slice());
  })
}

#[bench]
fn bench_fizzbuzz(b: &mut Bencher) {
    let prog = "n = 1

while n <= 100 {
  if n % 3 == 0 {
    if n % 5 == 0 {
      out 10
    } else {
      out 1
    }
  } else if n % 5 == 0 {
    out 0
  }
  n = n + 1
}
";
  let lexer = token();
  let (tokens, rest) = lexer.parse(prog).unwrap();
  let parser = program();
  b.iter(|| {
    parser.parse(tokens.as_slice());
  })
}

*/
  

