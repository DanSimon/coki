#![feature(phase)]
#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

use regex::{Captures, Regex};

pub struct parser;

type ParseResult<'a, T> = Result<(T, &'a str), String>;

trait Parser<'a, T> {

  fn parse(&self, data: &'a str) -> ParseResult<'a, T>;

  fn and_then<B>(&'a self, next: &'a Parser<'a, B>) -> DualParser<'a, T,B> {
    DualParser{first: self, second: next}
  }
  fn or<B>(&'a self, next: &'a Parser<'a, B>) -> OrParser<'a, T,B> {
    OrParser{a: self, b: next}
  }
}


pub struct CharParser{the_char: char}

impl CharParser {
  fn new(c: char) -> CharParser {
    CharParser{the_char: c}
  }
}

impl<'a> Parser<'a, char> for CharParser {

  fn parse(&self, data: &'a str) -> ParseResult<'a, char> {
    if data.len() > 0 {
      if data.char_at(0) == self.the_char {
        Ok((self.the_char, data.slice_from(1)))
      } else {
        Err(format!("Expected {}, got {}", self.the_char, data.char_at(0)))
      }
    } else {
      Err(format!("No data left"))
    }
  }
}

pub struct RegexParser {
  regex: Regex
}

impl<'a> Parser<'a, Captures<'a>> for RegexParser {
  fn parse(&self, data: &'a str) -> ParseResult<'a, Captures<'a>> {
    match self.regex.captures(data) {
      Some(cap) => {
        let start = cap.at(0).len();
        Ok((cap, data.slice_from(start)))
      }
      _ => Err(format!("no match or match not on first bye"))
    }
  }
}
  



pub struct RepParser<'a, T>{
  reps: uint,
  parser: &'a Parser<'a, T> + 'a
}

impl<'a, T> Parser<'a, Vec<T>> for RepParser<'a, T> {
  fn parse(&self, data: &'a str) -> ParseResult<'a, Vec<T>> {
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
  first: &'a Parser<'a, A> + 'a,
  second: &'a Parser<'a, B> + 'a
}

impl <'a, A, B> Parser<'a, (A,B)> for DualParser<'a, A, B> {
  
  fn parse(&self, data: &'a str) -> ParseResult<'a, (A, B)> {
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
  a: &'a Parser<'a, A> + 'a,
  b: &'a Parser<'a, B> + 'a
}

impl<'a, A, B> Parser<'a, Or<A,B>> for OrParser<'a, A, B> {
  fn parse(&self, data: &'a str) -> ParseResult<'a, Or<A, B>> {
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

#[test]
fn test_regex() {
  let reg = regex!("ab[cd]");
  let parser_a = RegexParser{regex: reg};
  let data = "abdabc";
  assert!(parser_a.parse(data) == Ok(("abd", "abc")));
}

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
  



  
    
