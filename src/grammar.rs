
#[deriving(Show)]
#[deriving(Clone)]
#[deriving(PartialEq)]
pub enum AddOp {
  Add,
  Subtract,
}

#[deriving(Show)]
#[deriving(Clone)]
#[deriving(PartialEq)]
pub enum MultOp {
  Multiply,
  Divide,
  Modulo,
}

#[deriving(Clone)]
#[deriving(Show)]
#[deriving(PartialEq)]
pub struct AddTerm(pub AddOp, pub Expr);

#[deriving(Clone)]
#[deriving(Show)]
#[deriving(PartialEq)]
pub struct MultTerm(pub MultOp, pub Expr);

#[deriving(Show)]
#[deriving(Clone)]
#[deriving(PartialEq)]
pub enum Expr {
  Variable(String),  
  Num(int),
  AddSub(Vec<AddTerm>), //a + b - c + d becomes [(+ a) (+ b) (- c) (+ d)]
  MultDiv(Vec<MultTerm>), 
}


//for now this is it's own type and not a statement
#[deriving(Show)]
#[deriving(Clone)]
#[deriving(PartialEq)]
pub struct Block(pub Vec<Statement>);

#[deriving(Show)]
#[deriving(Clone)]
#[deriving(PartialEq)]
pub enum Statement {
  Assign(String, Expr),
  Output(Expr),
  If(Expr, Comparator, Expr, Block, Option<Block>),
  While(Expr, Comparator, Expr, Block),
}

#[deriving(Show)]
#[deriving(Clone)]
#[deriving(Eq)]
#[deriving(PartialEq)]
pub enum Comparator {
  CEq,  // ==
  CGt,  // >
  CLt,  // <
  CNeq, // !=
  CGeq, // >=
  CLeq, // <=
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

