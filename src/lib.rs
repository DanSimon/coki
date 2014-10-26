extern crate regex;

use regex::Regex;

pub struct parser;

type ParseResult<'a, T> = Result<(T, &'a[u8]), String>;

trait Parser<T> {

  fn parse<'a>(&self, data: &'a [u8]) -> ParseResult<'a, T>;

  fn and_then<'a, B>(&'a self, next: &'a Parser<B>) -> DualParser<T,B> {
    DualParser{first: self, second: next}
  }
  fn or<'a, B>(&'a self, next: &'a Parser<B>) -> OrParser<T,B> {
    OrParser{a: self, b: next}
  }
}


pub struct CharParser{the_char: u8}

impl CharParser {
  fn new(c: u8) -> CharParser {
    CharParser{the_char: c}
  }
}

impl Parser<char> for CharParser {

  fn parse<'a>(&self, data: &'a [u8]) -> ParseResult<'a, char> {
    if data.len() > 0 {
      if (data[0] == self.the_char) {
        Ok((data[0] as char, data.slice_from(1)))
      } else {
        Err(format!("Expected {}, got {}", self.the_char, data[0]))
      }
    } else {
      Err(format!("No data left"))
    }
  }
}

pub struct RegexParser {
  regex: Regex
}
//TODO: this is all kinds of horrible, had a lot of trouble getting it to work
impl Parser<String> for RegexParser {
  fn parse<'a>(&self, data: &'a [u8]) -> ParseResult<'a, String> {
    let s = String::from_utf8_lossy(data);
    let strdata = s.as_slice();
    match self.regex.find(strdata) {
      Some((0, e)) => Ok((String::from_str(strdata.slice_to(e)), data.slice_from(e))),
      _ => Err(format!("no match or match not on first bye"))
    }
  }
}
  



pub struct RepParser<'a, T>{
  reps: uint,
  parser: &'a Parser<T> + 'a
}

impl<'a, T> Parser<Vec<T>> for RepParser<'a, T> {
  fn parse<'a>(&self, data: &'a [u8]) -> ParseResult<'a, Vec<T>> {
    let mut v: Vec<T> = Vec::new();
    let mut remain = data;
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

pub struct DualParser<'a, A, B> {
  first: &'a Parser<A> + 'a,
  second: &'a Parser<B> + 'a
}

impl <'a, A, B> Parser<(A,B)> for DualParser<'a, A, B> {
  
  fn parse<'a>(&self, data: &'a [u8]) -> ParseResult<'a, (A, B)> {
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

pub enum Or<A,B> {
  OrA(A),
  OrB(B),
}

pub struct OrParser<'a, A, B> {
  a: &'a Parser<A> + 'a,
  b: &'a Parser<B> + 'a
}

impl<'a, A, B> Parser<Or<A,B>> for OrParser<'a, A, B> {
  fn parse<'a>(&self, data: &'a [u8]) -> ParseResult<'a, Or<A, B>> {
    match self.a.parse(data) {
      Ok((a, d2)) => Ok((OrA(a), d2)),
      Err(err) => match self.b.parse(data) {
        Ok((b, remain)) => Ok((OrB(b), remain)),
        Err(err) => Err(err)
      }
    }
  }
}
  
  
  

  

#[test]
fn test_char() {
  let ch = CharParser::new('v' as u8);
  let data = ['v' as u8, 'b'as u8, 'x' as u8];
  match ch.parse(&data) {
    Ok((c, rest)) => {
      assert!(c == 'v');
      assert!(rest.len() == 2);
      assert!(rest[0] == 'b' as u8);
    }
    Err(err) => {fail!(format!("unepected error: {}", err));}
  }
}

#[test]
fn test_rep() {
  let ch = CharParser::new('v' as u8);
  let rep = RepParser{reps: 3, parser: &ch};
  let data = ['v' as u8, 'v'as u8, 'v' as u8, 'x' as u8];
  match rep.parse(&data) {
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
  let a = CharParser::new('a' as u8);
  let b = CharParser::new('b' as u8);
  let ab = a.and_then(&b);
  let data = ['a' as u8, 'b'as u8, 'v' as u8, 'x' as u8];
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
  let a = CharParser::new('a' as u8);
  let b = CharParser::new('b' as u8);
  let ab = a.or(&b);
  let data = ['a' as u8, 'b'as u8, 'v' as u8, 'x' as u8];
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
  




  
    
