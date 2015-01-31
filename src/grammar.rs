
#[derive(Show)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum AddOp {
  Add,
  Subtract,
}

#[derive(Show)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum MultOp {
  Multiply,
  Divide,
  Modulo,
}

#[derive(Clone)]
#[derive(Show)]
#[derive(PartialEq)]
pub struct AddTerm(pub AddOp, pub Expr);

#[derive(Clone)]
#[derive(Show)]
#[derive(PartialEq)]
pub struct MultTerm(pub MultOp, pub Expr);

#[derive(Show)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum Expr {
  Variable(String),  
  Num(i32),
  AddSub(Vec<AddTerm>), //a + b - c + d becomes [(+ a) (+ b) (- c) (+ d)]
  MultDiv(Vec<MultTerm>), 
}


//for now this is it's own type and not a statement
#[derive(Show)]
#[derive(Clone)]
#[derive(PartialEq)]
pub struct Block(pub Vec<Statement>);

#[derive(Show)]
#[derive(Clone)]
#[derive(PartialEq)]
pub enum Statement {
  Assign(String, Expr),
  Output(Expr),
  If(Expr, Comparator, Expr, Block, Option<Block>),
  While(Expr, Comparator, Expr, Block),
}

#[derive(Show)]
#[derive(Clone)]
#[derive(Eq)]
#[derive(PartialEq)]
pub enum Comparator {
  CEq,  // ==
  CGt,  // >
  CLt,  // <
  CNeq, // !=
  CGeq, // >=
  CLeq, // <=
}



#[derive(Show)]
#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum Token {
  Equals,
  Ident(String),
  Number(i32),
  PlusSign,
  MinusSign,
  MultSign,
  DivideSign,
  ModuloSign,
  OutputCmd,
  NewLine,
  OpenParen,
  CloseParen,
  OpenBrace,
  CloseBrace,
  IfKeyword,
  ElseKeyword,
  WhileKeyword,
  Cmp(Comparator),
}

