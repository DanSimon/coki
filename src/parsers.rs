
use std::ops::Fn;
use regex::{Captures, Regex};


pub fn literal<'a, T:'a + Eq + Clone>(literal: T) -> Box<Parser<'a, &'a[T], T> + 'a> {
  box LiteralParser{literal: literal}
}

pub type ParseResult<'a, I:'a, O> = Result<(O, I), String>;


pub trait Parser<'a, I, O> {

  fn parse(&self, data: I) -> ParseResult<'a, I, O>;

}


pub struct LiteralParser<'a, T:'a + Eq> {
  pub literal: T,
}

impl<'a, T: 'a + Eq + Clone> Parser<'a,  &'a [T], T> for LiteralParser<'a, T> {
  fn parse(&self, data: &'a[T]) -> ParseResult<'a, &'a[T], T> {
    if data.len() < 1 {
      return Err(format!("ran out of data"))
    }
    if data[0] == self.literal {
      Ok((data[0].clone(), data.slice_from(1)))
    } else {
      Err(format!("Literal mismatch"))
    }
  }
}

pub struct RegexLiteralParser<'a> {
  pub regex: Regex,
}

impl<'a> Parser<'a, &'a str, ()> for RegexLiteralParser<'a> {
  fn parse(&self, data: &'a str) -> ParseResult<'a, &'a str, ()> {
    self.regex.find(data).map(|(_, e)| ((), data.slice_from(e))).ok_or(format!("regex literal match fail"))
  }
}

pub struct RegexCapturesParser<'a> {
  pub regex: Regex,
}
impl<'a> Parser<'a, &'a str, Captures<'a>> for RegexCapturesParser<'a> {
  fn parse(&self, data: &'a str) -> ParseResult<'a, &'a str, Captures<'a>> {
    match self.regex.captures(data) {
      Some(caps) => match caps.pos(0) {
        Some((_, e)) => Ok((caps, data.slice_from(e))),
        None => Err(format!("No Match"))
      },
      None => Err(format!("No Match"))
    }
  }
}





pub struct MatchParser<'a, I, O> {
  pub matcher: Box< Fn<(&'a I,), Result<O, String>> +'a>
}
impl<'a, I: Clone, O> Parser<'a, &'a [I], O> for MatchParser<'a, I, O> {
  fn parse(&self, data: &'a[I]) -> ParseResult<'a, &'a[I], O> {
    if data.len() < 1 {
      Err(format!("Unexpected End!"))
    } else {
      self.matcher.call((&data[0],)).map(|res| (res, data.slice_from(1)))
    }
  }
}

    

pub struct RepParser<'a, I, O>{
  pub parser: Box<Parser<'a, I, O> + 'a>
}

impl<'a, I: Clone, O> Parser<'a, I, Vec<O>> for RepParser<'a, I, O> {
  fn parse(&self, data: I) -> ParseResult<'a, I, Vec<O>> {
    let mut remain = data;
    let mut v: Vec<O> = Vec::new();
    loop {
      match self.parser.parse(remain.clone()) {
        Ok((result, rest)) => {
          v.push(result);
          remain = rest;
        }
        Err(_) => {
          return Ok((v, remain));
        }
      }
    }
  }
}

//sep should return a Failure when it's time to stop repeating, the ok value is never used
pub struct RepSepParser<'a, I, O, U> {
  pub rep: Box<Parser<'a, I, O> + 'a>,
  pub sep: Box<Parser<'a, I, U> + 'a>,
  pub min_reps: uint,
}
impl<'a, I: Clone, O, U> Parser<'a, I, Vec<O>> for RepSepParser<'a, I, O, U> {
  fn parse(&self, data: I) -> ParseResult<'a, I, Vec<O>> {
    let mut remain = data;
    let mut v: Vec<O> = Vec::new();    
    loop {
      match self.rep.parse(remain) {
        Ok((result, rest)) => {
          v.push(result);
          match self.sep.parse(rest.clone()) {
            Ok((_, rest2)) => {
              remain = rest2
            }
            Err(_) => {
              if v.len() < self.min_reps {
                return Err(format!("Not enough reps: required {}, got {}", self.min_reps, v.len()))
              } else {
                return Ok((v, rest)) 
              }
            }
          }
        }
        Err(err) => {
          return Err(format!("Error on rep: {}", err));
        }
      }
    }
    //?
  }
}
  


pub struct DualParser<'a, I, A, B> {
  pub first: Box<Parser<'a, I, A> + 'a>,
  pub second: Box<Parser<'a, I, B> + 'a>
}

impl <'a, I, A, B> Parser<'a, I, (A,B)> for DualParser<'a, I, A, B> {
  
  fn parse(&self, data: I) -> ParseResult<'a, I, (A, B)> {
    match self.first.parse(data) {
      Ok((a, d2)) => match self.second.parse(d2) {
        Ok((b, remain)) => Ok(((a, b), remain)),
        Err(err) => Err(err)
      },
      Err(err) => Err(err)
    }
  }
}

//we need lazy evaluation to be able to support recursive grammars!
pub struct OrParser<'a, I, O> {
  pub a: Box<Fn<(), Box<Parser<'a, I, O> + 'a> + 'a> + 'a>,
  pub b: Box<Fn<(), Box<Parser<'a, I, O> + 'a> + 'a> + 'a>
}

/*
 * Notice that I needs to be cloneable because we have to be able to hand it off to each parser
 */
impl<'a, I: Clone, O> Parser<'a, I, O> for OrParser<'a, I, O> {
  fn parse(&self, data: I) -> ParseResult<'a, I, O> {
    match self.a.call(()).parse(data.clone()) {
      Ok((a, d2)) => Ok((a, d2)),
      Err(_) => match self.b.call(()).parse(data.clone()) {
        Ok((b, remain)) => Ok((b, remain)),
        Err(err) => Err(err)
      }
    }
  }
}

pub struct OneOfParser<'a, I, O> {
  parsers: Vec<Box<Fn<(), Box<Parser<'a, I, O> + 'a> + 'a> + 'a> + 'a>
}

impl<'a, I: Clone, O> Parser<'a, I, O> for OneOfParser<'a, I, O> {
  fn parse(&self, data: I) -> ParseResult<'a, I, O> {
    for p in self.parsers.iter() {
      let res = p.call(()).parse(data.clone());
      if res.is_ok() {
        return res;
      }
    }
    Err(format!("All options failed"))
  }
}

pub struct MapParser<'a, I, O, U> {
  pub parser: Box<Parser<'a, I, O> + 'a>,
  pub mapper: Box<Fn<(O,), U> +'a>, //this has to be a &Fn and not a regular lambda since it must be immutable
}
impl<'a, I, O, U> Parser<'a, I, U> for MapParser<'a, I, O, U> {
  fn parse(&self, data: I) -> ParseResult<'a, I, U> {
    self.parser.parse(data).map(|(output, input)| ((self.mapper.call((output,)), input)))
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
    
