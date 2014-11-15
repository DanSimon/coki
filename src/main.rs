#![feature(globs)]
#![feature(phase)]
#![feature(unboxed_closures)]
#![feature(macro_rules)]
extern crate regex;
#[phase(plugin)] extern crate peruse;
extern crate peruse;
extern crate test;

use std::mem::replace;
use test::Bencher;

use std::collections::HashMap;
use peruse::parsers::*;
use grammar::*;
use parser::program;
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
      interp(raw.as_slice());
    }
    Err(err) => {println!("Error Reading File: {}", err);}
  }

}

type Environment = HashMap<String, int>;

fn interp<'a>(raw: &'a str) {
  let lexer = token();
  match lexer.parse(raw) {
    Ok((tokens, rest)) => {
      if rest != "" {
        println!("Parser error at: {}", rest)
      } else {
        let parser = program();
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
          if (is_true) {            
            let &Block(ref p) = then_block;
            run_internal(p, env);
          } else {
            match else_block {
              &Some(Block(ref p)) => {run_internal(p, env);}
              &None => {}
            }
          }
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
            Multiply  => {total *= value;}
            Divide    => {total /= value;}
            Modulo    => {total %= value;} 
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
  

