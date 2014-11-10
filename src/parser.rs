use std::fmt::Show;
use peruse::parsers::*;
use grammar::*;

pub type LParser<'a, T> = Box<Parser<'a, &'a [Token], T> + 'a>;


fn assign<'a>() -> LParser<'a, Statement> {
  map!( 
    seq!(variable(), literal(Equals), expr()),
    |&: (var,( _, expr))| match var{
      Variable(name) => Assign(name, expr),
      _ => unreachable!()
    }
  )
}

fn output<'a>() -> LParser<'a, Statement> {
  map!(
    seq!(literal(OutputCmd), expr()),
    |&: (_, var): (Token, Expr)| Output(var)
  )
}

fn if_stmt<'a>() -> LParser<'a, Statement> {
  //todo: optional else
  map!(
    seq!(literal(IfKeyword), expr(), comparator(), expr(), braced_block(), literal(ElseKeyword), braced_block()),
    |&: (_, (lhs, (comp, (rhs, (then_block, (_ , else_block))))))| If(lhs, comp, rhs, then_block, else_block)
  )
}

fn while_stmt<'a>() -> LParser<'a, Statement> {
  map!(
    seq!(literal(WhileKeyword), expr(), comparator(), expr(), braced_block()),
    |&: (_, (lhs, (comp, (rhs, block))))| While(lhs, comp, rhs, block)
  )
}
  

pub fn block<'a>() -> LParser<'a, Block> {
  map!(
    rep!(
      map!(
        seq!(
          or!(output(), if_stmt(), while_stmt(), assign()),
          literal(NewLine)
        ), 
        |&: (stmt, _)| stmt
      )
    ),
    |&: stmts| Block(stmts)
  )

}

pub fn braced_block<'a>() -> LParser<'a, Block> {
  map!(
    seq!(literal(OpenBrace), block(), literal(CloseBrace)),
    |&: (_, (block, _))| block
  )
}

pub fn comparator<'a>() -> LParser<'a, Comparator> {
  box MatchParser{
    matcher: box |&: input: &Token| match *input {
      Cmp(c) => Ok(c),
      ref other => Err(format!("Expected comparator, got {}", other))
    }
  }
}

  


fn variable<'a>() -> LParser<'a, Expr> {
  box MatchParser{
    matcher: box |&: input: &Token| match input {
      &Ident(ref str) => Ok(Variable(str.clone())),
      other => Err(format!("Expected variable, got {}", other))
    }
  }
}

type EParser<'a> = LParser<'a, Expr>;

fn expr<'a>() -> EParser<'a> {

  fn number<'a>() -> EParser<'a> {
    box MatchParser{matcher: box |&: input: &Token| -> Result<Expr, String> match *input {
      Number(num) => Ok(Num(num)),
      ref other => Err(format!("wrong type, expected number, got {}", other))
    }}
  }

  fn term<'a>() -> EParser<'a> {
    or!(paren_expr(), variable(), number())
  }

  fn mult<'a>() -> EParser<'a> {
    map!(
      seq!(
        term(),
        rep!(seq!(or!(literal(MultSign), literal(DivideSign)), term()))
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
              _ => panic!("not allowed")
            };
            f.push(MultTerm(s, value.clone())); //maybe box the value instead
          }
          MultDiv(f)
        }
      }
    )
  }

  fn simple_expr<'a>() -> EParser<'a> {
    or!(mult(), term()) 
  }

  fn plus<'a>() -> EParser<'a> {
    map!(
      seq!(
        simple_expr(),
        rep!(seq!(or!(literal(PlusSign), literal(MinusSign)), simple_expr()))
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
  }

  fn paren_expr<'a>() -> EParser<'a> {
    map!(
      seq!(literal(OpenParen), expr(), literal(CloseParen)),
      |&: (_, (expr, _))| expr
    )
  }

  plus()
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
  let parser = assign();
  let input = [Ident(from_str("x").unwrap()), Equals, Number(7)];
  let expected = Assign(from_str("x").unwrap(), Num(7));
  test_parser(input.as_slice(), &*parser, expected);
}

#[test]
fn test_simple_output() {
  let parser = output();
  let input = [OutputCmd, Number(4)];
  let expected = Output(Num(4));
  test_parser(input.as_slice(), &*parser, expected);
}
