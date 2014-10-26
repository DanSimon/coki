use std::collections::HashMap;

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

}

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

enum Token {
  Equals,
  Ident(String),
  Number(int),
  PlusSign,
  OutputCmd,
  NewLine,
}
