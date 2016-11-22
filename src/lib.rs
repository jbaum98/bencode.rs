#[macro_use]
extern crate nom;
use nom::{IResult, digit};
pub use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ParserError {
    data: String
}
impl std::fmt::Display for ParserError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt, "{}", self.data)
}
}
impl std::error::Error for ParserError {
    fn description(&self) -> &str {
        &self.data
    }
}
impl<'a> From<nom::Err<&'a [u8]>> for ParserError {
    fn from(e: nom::Err<&'a [u8]>) -> ParserError {
        ParserError {
            data: format!("Error: {:?}", e)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum BVal {
    BInt(i64),
    BStr(String),
    BList(Vec<BVal>),
    BDict(BTreeMap<String, BVal>),
}

fn digits<O>(input: &[u8]) -> IResult<&[u8], O>
where O: std::str::FromStr {
    map_res!(input, map_res!(digit, std::str::from_utf8), std::str::FromStr::from_str)
}

fn num<O: std::ops::Neg<Output=O>>(input: &[u8]) -> IResult<&[u8], O>
where O: std::str::FromStr {
    chain!(input,
        char!('i') ~
        neg: char!('-')? ~
        n: digits ~
        char!('e') ,
        ||{
            let n:O = n;
            if neg.is_some() {n.neg()} else {n}
        })
}


fn string(input: &[u8]) -> IResult<&[u8], String> {
    let parse_len: IResult<&[u8], usize> =
        chain!(input,
               len: digits ~
               char!(':') ,
               || {len}
        );

    match parse_len {
        IResult::Done(left, len)    => map_res!(left, map!(take!(len), |s: &[u8]| {s.to_vec()}), String::from_utf8),
        IResult::Incomplete(needed) => IResult::Incomplete(needed),
        IResult::Error(err)         => IResult::Error(err),
    }
}

named!(list< Vec<BVal> >, delimited!(char!('l'), many0!(bval), char!('e')));
named!(dict< BTreeMap<String, BVal> >,
       delimited!(
           char!('d'),
           fold_many0!(
               pair!(string, bval),
               BTreeMap::new(),
               |mut map: BTreeMap<String,BVal>, (k,v)| {
                   map.insert(k,v);
                   map
               }),
           char!('e')
       )
);

named!(bnum<BVal>, map!(num, BVal::BInt));
named!(bstring<BVal>, map!(string, BVal::BStr));
named!(blist<BVal>, map!(list, BVal::BList));
named!(bdict<BVal>, map!(dict, BVal::BDict));
named!(bval<BVal>, alt!(bnum | bstring | blist | bdict));

#[cfg(test)]
mod tests {
    use nom::IResult;
    use nom::IResult::*;
    use super::BVal::*;
    use super::BTreeMap;

    fn parse<I, O, F>(parser: F, input: I) -> O
        where F: Fn(I) -> IResult<I,O> {
        match parser(input) {
            Done(_, n) => n,
            Incomplete(i) => panic!(format!("Incomplete: {:?}", i)),
            _ => panic!("Error while parsing number"),
        }
    }

    fn parse_num(input: &[u8]) -> i64 { parse(super::num, input) }

    #[test]
    fn test_num_1digit() {
        assert_eq!(1, parse_num(b"i1e"));
    }

    #[test]
    fn test_num_2digit() {
        assert_eq!(123, parse_num(b"i123e"));
    }

    #[test]
    fn test_num_negative() {
        assert_eq!(-6, parse_num(b"i-6e"));
    }

    #[test]
    fn test_string() {
        assert_eq!("Hello!".to_string(), parse(super::string, b"6:Hello!"));
    }

    #[test]
    fn test_bval_num() {
        assert_eq!(BInt(-300), parse(super::bval, b"i-300e"));
    }

    #[test]
    fn test_bval_string() {
        assert_eq!(BStr("Hello!".to_string()), parse(super::bval, b"6:Hello!"));
    }

    #[test]
    fn test_bval_list() {
        let list = BList(vec![BInt(0), BStr("Hello!".to_string()), BInt(2)]);
        assert_eq!(list, parse(super::bval, b"li0e6:Hello!i2ee"));
    }

    #[test]
    fn test_bval_dict() {
        let mut dict = BTreeMap::new();
        dict.insert("Pears".to_string(), BInt(5));
        dict.insert("Apples".to_string(), BInt(-4));
        dict.insert("Bananas".to_string(), BList(vec![BInt(5), BInt(2), BStr(":(".to_string())]));
        let dict = BDict(dict);
        assert_eq!(dict, parse(super::bval, b"d6:Applesi-4e7:Bananasli5ei2e2::(e5:Pearsi5ee"));
    }
}
