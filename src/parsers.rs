#![feature(phase)]
#[phase(plugin)]

//use regex::{Captures, Regex};

pub struct parser;

pub type ParseResult<'a, I:'a, O> = Result<(O, I), String>;

pub trait Parser<'a, I, O> {

  fn parse(&self, data: I) -> ParseResult<'a, I, O>;
  fn and_then<B>(&'a self, next: &'a Parser<'a, I, B>) -> DualParser<'a, I, O ,B> {
    DualParser{first: self, second: next}
  }
  fn or<B>(&'a self, next: &'a Parser<'a, I, B>) -> OrParser<'a, I, O, B> {
    OrParser{a: self, b: next}
  }
}

trait TokenParser<'a, I, O> : Parser<'a, Vec<I>, O> {

}

pub struct LiteralParser<'a, T:'a + Eq> {
  pub literal: T,
}

impl<'a, T: 'a + Eq + Clone> Parser<'a,  &'a [T], T> for LiteralParser<'a, T> {
  fn parse(&self, data: &'a[T]) -> ParseResult<'a, &'a[T], T> {
    if data[0] == self.literal {
      Ok((data[0].clone(), data.slice_from(1)))
    } else {
      Err(format!("Literal mismatch"))
    }
  }
}

/*
pub struct MatchParser<'a, I, O> {
  matcher: |I| -> Option<O>
}
impl
*/


pub struct RepParser<'a, I, O>{
  pub reps: uint,
  pub parser: &'a Parser<'a, I, O> + 'a
}

impl<'a, I, O> Parser<'a, I, Vec<O>> for RepParser<'a, I, O> {
  fn parse(&self, data: I) -> ParseResult<'a, I, Vec<O>> {
    let mut remain = data;
    let mut v: Vec<O> = Vec::new();
    for i in range(0, self.reps) {
      match self.parser.parse(remain) {
        Ok((result, rest)) => {
          v.push(result);
          remain = rest;
        }
        Err(err) => {
          return Err(format!("Error on rep #{}: {}", i, err));
        }
      }
    }
    Ok((v, remain))
  }
}


pub struct DualParser<'a, I, A, B> {
  first: &'a Parser<'a, I, A> + 'a,
  second: &'a Parser<'a, I, B> + 'a
}

impl <'a, I, A, B> Parser<'a, I, (A,B)> for DualParser<'a, I, A, B> {
  
  fn parse(&self, data: I) -> ParseResult<'a, I, (A, B)> {
    /*  doesn't work :(
    self.first.parse(data).and_then(
      |(a, d2)| self.second.parse(d2).and_then(
        |(b, remain)| Ok(((a, b), remain))
      )
    )
    */
    match self.first.parse(data) {
      Ok((a, d2)) => match self.second.parse(d2) {
        Ok((b, remain)) => Ok(((a, b), remain)),
        Err(err) => Err(err)
      },
      Err(err) => Err(err)
    }
  }
}

#[deriving(Show)]
pub enum Or<A,B> {
  OrA(A),
  OrB(B),
}

pub struct OrParser<'a, I, A, B> {
  pub a: &'a Parser<'a, I, A> + 'a,
  pub b: &'a Parser<'a, I, B> + 'a
}

/*
 * Notice that I needs to be cloneable because we have to be able to hand it off to each parser
 */
impl<'a, I: Clone, A, B> Parser<'a, I, Or<A,B>> for OrParser<'a, I, A, B> {
  fn parse(&self, data: I) -> ParseResult<'a, I, Or<A, B>> {
    match self.a.parse(data.clone()) {
      Ok((a, d2)) => Ok((OrA(a), d2)),
      Err(err) => match self.b.parse(data.clone()) {
        Ok((b, remain)) => Ok((OrB(b), remain)),
        Err(err) => Err(err)
      }
    }
  }
}

pub struct OneOfParser<'a, I, O> {
  parsers: &'a [&'a Parser<'a, I, O> + 'a]
}

impl<'a, I: Clone, O> Parser<'a, I, O> for OneOfParser<'a, I, O> {
  fn parse(&self, data: I) -> ParseResult<'a, I, O> {
    for p in self.parsers.iter() {
      let res = p.parse(data.clone());
      if res.is_ok() {
        return res;
      }
    }
    Err(format!("All options failed"))
  }
}
      
      


  
/*
  

#[test]
fn test_char() {
  let ch = CharParser::new('v');
  let data = "vbx";
  match ch.parse(data) {
    Ok((c, rest)) => {
      assert!(c == 'v');
      assert!(rest.len() == 2);
      assert!(rest.char_at(0) == 'b');
    }
    Err(err) => {fail!(format!("unepected error: {}", err));}
  }
}

/*
#[test]
fn test_regex() {
  let reg = regex!("ab[cd]");
  let parser_a = RegexParser{regex: reg};
  let data = "abdabc";
  assert!(parser_a.parse(data) == Ok(("abd", "abc")));
}
*/

#[test]
fn test_rep() {
  let ch = CharParser::new('v');
  let rep = RepParser{reps: 3, parser: &ch};
  let data = "vvvx";
  match rep.parse(data) {
    Ok((vec, rest)) => {
      assert!(vec.len() == 3)
      for c in vec.iter() {
        assert!(*c == 'v');
      }
    }
    Err(err) => {fail!(format!("unepected error: {}", err));}
  }
}

#[test]
fn test_and_then() {
  let a = CharParser::new('a' );
  let b = CharParser::new('b' );
  let ab = a.and_then(&b);
  let data = "abvx";
  match ab.parse(data) {
    Ok(((a, b), rem)) => {
      assert!(a == 'a');
      assert!(b == 'b');
      assert!(rem.len() == 2);
    }
    Err(err) => {
      fail!(err);
    }
  }
}

#[test]
fn test_or() {
  let a = CharParser::new('a' );
  let b = CharParser::new('b' );
  let ab = a.or(&b);
  let data = "abvx";
  match ab.parse(data) {
    Ok((OrA('a'), rem)) => {
      match ab.parse(rem) {
        Ok((OrB('b'), rem)) => {
          assert!(rem.len() == 2);
        }
        _ => {fail!("wrong b");}
      }
    }
    _ => {
      fail!("wrong a");
    }
  }
}
  



 */ 
    
