use std::fmt::Show;
use peruse::parsers::*;
use grammar::*;

pub type LParser<'a, T> = Box<Parser<'a, &'a [Token], T> + 'a>;

macro_rules! variable { () => {
  MatchParser{
    matcher: box |&: input: &Token| match input {
      &Ident(ref str) => Ok(Variable(str.clone())),
      other => Err(format!("Expected variable, got {}", other))
    }
  }
}}

macro_rules! assign { () => {
  map!( 
    seq!(variable!(), literal(Equals), lazy!(expr())),
    |&: (var,( _, expr))| match var{
      Variable(name) => Assign(name, expr),
      _ => unreachable!()
    }
  )
}}

macro_rules! output{ () => {
  map!(
    seq!(literal(OutputCmd), lazy!(expr())),
    |&: (_, var): (Token, Expr)| Output(var)
  )
}}

macro_rules! block { () => {
  map!(
    rep!(
      map!(
        seq!(
          or!(output!(), lazy!(if_stmt()), lazy!(while_stmt()), assign!()),
          literal(NewLine)
        ), 
        |&: (stmt, _)| stmt
      )
    ),
    |&: stmts| Block(stmts)
  )

}}

macro_rules! braced_block { () => {
  map!(
    seq!(literal(OpenBrace), block!(), literal(CloseBrace)),
    |&: (_, (block, _))| block
  )
}}

macro_rules! comparator { () => {
  MatchParser{
    matcher: box |&: input: &Token| match *input {
      Cmp(c) => Ok(c),
      ref other => Err(format!("Expected comparator, got {}", other))
    }
  }
}}

fn if_stmt<'a>() -> LParser<'a, Statement> {
  //todo: optional else
  box map!(
    seq!(
      literal(IfKeyword), 
      lazy!(expr()), 
      comparator!(), 
      lazy!(expr()), 
      braced_block!(), 
      opt!(map!(
        seq!(
          literal(ElseKeyword), 
          or!(
            braced_block!(), 
            map!(lazy!(if_stmt()), |&: if_stmt| Block(vec![if_stmt])) //else if...
          )
        ),
        |&: (_, else_block)| else_block
      ))
    ),
    |&: (_, (lhs, (comp, (rhs, (then_block, else_block_opt)))))| If(lhs, comp, rhs, then_block, else_block_opt)
  )
}

fn while_stmt<'a>() -> LParser<'a, Statement> {
  box map!(
    seq!(literal(WhileKeyword), lazy!(expr()), comparator!(), lazy!(expr()), braced_block!()),
    |&: (_, (lhs, (comp, (rhs, block))))| While(lhs, comp, rhs, block)
  )
}
  


  

pub fn program<'a>() -> LParser<'a, Block> {
  box block!()
}


type EParser<'a> = LParser<'a, Expr>;

fn expr<'a>() -> EParser<'a> {

  macro_rules! number { () => { MatchParser{matcher: box |&: input: &Token| -> Result<Expr, String> match *input {
    Number(num) => Ok(Num(num)),
    ref other => Err(format!("wrong type, expected number, got {}", other))
  }}}}

  macro_rules! paren_expr  { () => {
    map!(
      seq!(literal(OpenParen), lazy!(expr()), literal(CloseParen)),
      |&: (_, (expr, _))| expr
    )
  }}

  macro_rules! term { () =>  {or!(paren_expr!(), variable!(), number!())}}

  let mult = || {
    map!(
      seq!(
        term!(),
        rep!(seq!(or!(literal(MultSign), literal(DivideSign), literal(ModuloSign)), term!()))
      ),
      |&: (first, rest): (Expr, Vec<(Token, Expr)>)| {
        if rest.len() == 0 {
          first
        } else {
          let mut f = Vec::new();
          f.push(MultTerm(Multiply, first));
          for &(ref sign, ref value) in rest.iter() {
            let s = match *sign {
              MultSign => Multiply,
              DivideSign => Divide,
              ModuloSign => Modulo,
              _ => unreachable!()
            };
            f.push(MultTerm(s, value.clone())); //maybe box the value instead
          }
          MultDiv(f)
        }
      }
    )
  };

  
  macro_rules! simple_expr { () => { or!(mult(), term!()) }}

  let plus = {
    map!(
      seq!(
        simple_expr!(),
        rep!(seq!(or!(literal(PlusSign), literal(MinusSign)), simple_expr!()))
      ),
      |&: (first, rest): (Expr, Vec<(Token, Expr)>)| {
        if rest.len() == 0 {
          first
        } else {
          let mut f = Vec::new();
          f.push(AddTerm(Add, first));
          for &(ref sign, ref value) in rest.iter() {
            let s = match *sign {
              PlusSign => Add,
              MinusSign => Subtract,
              _ => panic!("not allowed")
            };
            f.push(AddTerm(s, value.clone()));
          }
          AddSub(f)
        }
      }
    )
  };


  box plus
}

fn test_parser<'a, I, O: PartialEq + Show>(input: I, parser: &Parser<'a, I, O>, expected: O) {
  match parser.parse(input) {
    Ok((output, rest)) => {
      assert_eq!(output, expected);
    },
    Err(err) => panic!(err)
  }
}

#[test]
fn test_term() {
  let parser = expr();
  let input = [Number(5)];
  let expected = Num(5);
  test_parser(input.as_slice(), &*parser, expected);
}

#[test]
fn test_plus_sequence() {
  let parser = expr();
  let input = [Number(5), PlusSign, Number(3)];
  let expected = AddSub(vec![AddTerm(Add, Num(5)), AddTerm(Add, Num(3))]);
  test_parser(input.as_slice(), &*parser, expected);
}

#[test]
fn test_plus_mult_sequence() {
  let parser = expr();
  let input = [Number(3), PlusSign, Number(4), MultSign, Number(5)];
  let expected = AddSub(vec![AddTerm(Add, Num(3)), AddTerm(Add, MultDiv(vec![MultTerm(Multiply, Num(4)), MultTerm(Multiply, Num(5))]))]);
  test_parser(input.as_slice(), &*parser, expected);
}


#[test]
fn test_simple_assign() {
  let parser = assign!();
  let input = [Ident(from_str("x").unwrap()), Equals, Number(7)];
  let expected = Assign(from_str("x").unwrap(), Num(7));
  test_parser(input.as_slice(), &parser, expected);
}

#[test]
fn test_simple_output() {
  let parser = output!();
  let input = [OutputCmd, Number(4)];
  let expected = Output(Num(4));
  test_parser(input.as_slice(), &parser, expected);
}
