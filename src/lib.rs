#[macro_use]
extern crate nom;

use nom::{IResult, digit};
use nom::IResult::*;
use std::collections::BTreeMap;
use std::ops::Neg;
use std::str;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum BVal {
    BInt(i64),
    BStr(String),
    BList(Vec<BVal>),
    BDict(BTreeMap<String, BVal>),
}

fn digits<O: FromStr>(input: &[u8]) -> IResult<&[u8], O> {
    map_res!(input, map_res!(digit, str::from_utf8), FromStr::from_str)
}

fn num<O: Neg<Output = O> + FromStr>(input: &[u8]) -> IResult<&[u8], O> {
    chain!(input,
        char!('i')       ~
        neg: char!('-')? ~
        n: digits        ~
        char!('e')       ,
        ||{
            let n: O = n;
            if neg.is_some() {n.neg()} else {n}
        })
}

fn string(input: &[u8]) -> IResult<&[u8], String> {
    let parse_len: IResult<&[u8], usize> = chain!(input,
               len: digits ~
               char!(':')  ,
               || {len}
        );

    match parse_len {
        Done(left, len) => {
            map_res!(left,
                     map!(take!(len), |s: &[u8]| s.to_vec()),
                     String::from_utf8)
        }
        Incomplete(needed) => Incomplete(needed),
        Error(err) => Error(err),
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

named!(bnum<BVal>,    map!(num,    BVal::BInt));
named!(bstring<BVal>, map!(string, BVal::BStr));
named!(blist<BVal>,   map!(list,   BVal::BList));
named!(bdict<BVal>,   map!(dict,   BVal::BDict));

named!(bval<BVal>, alt!(bnum | bstring | blist | bdict));

pub fn parse_bencode(input: &[u8]) -> Option<BVal> {
    match bval(input) {
        Done(_, val) => Some(val),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    pub use nom::IResult;
    pub use nom::IResult::*;

    pub fn parse<I, O, F>(parser: F, input: I) -> O
        where F: Fn(I) -> IResult<I, O>
    {
        match parser(input) {
            Done(_, n) => n,
            Incomplete(i) => panic!(format!("Incomplete: {:?}", i)),
            _ => panic!("Error while parsing"),
        }
    }

    mod num {
        use super::parse;
        use super::super::num;

        fn parse_num(input: &[u8]) -> i64 {
            parse(num, input)
        }

        #[test]
        fn digit1() {
            assert_eq!(1, parse_num(b"i1e"));
        }

        #[test]
        fn digit2() {
            assert_eq!(123, parse_num(b"i123e"));
        }

        #[test]
        fn negative() {
            assert_eq!(-6, parse_num(b"i-6e"));
        }
    }

    #[test]
    fn string() {
        assert_eq!("Hello!".to_string(), parse(super::string, b"6:Hello!"));
    }

    mod bval {
        use std::collections::BTreeMap;
        use super::parse;
        use super::super::BVal::*;
        use super::super::bval;

        #[test]
        fn bval_num() {
            assert_eq!(BInt(-300), parse(bval, b"i-300e"));
        }

        #[test]
        fn bval_string() {
            assert_eq!(BStr("Hello!".to_string()), parse(bval, b"6:Hello!"));
        }

        #[test]
        fn bval_list() {
            let list = BList(vec![BInt(0), BStr("Hello!".to_string()), BInt(2)]);
            assert_eq!(list, parse(bval, b"li0e6:Hello!i2ee"));
        }

        #[test]
        fn bval_dict() {
            let mut dict = BTreeMap::new();
            dict.insert("Pears".to_string(), BInt(5));
            dict.insert("Apples".to_string(), BInt(-4));
            dict.insert("Bananas".to_string(),
                        BList(vec![BInt(5), BInt(2), BStr(":(".to_string())]));
            let dict = BDict(dict);
            assert_eq!(dict, parse(bval, b"d6:Applesi-4e7:Bananasli5ei2e2::(e5:Pearsi5ee"));
        }
    }
}
