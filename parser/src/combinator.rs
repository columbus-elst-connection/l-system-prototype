use std::marker::PhantomData;

pub trait Parser<T> {
    fn parse<'a>(&self, input: &'a str) -> Result<(T, &'a str), ParseError>;
}

impl <T, F> Parser<T> for F where F: for <'a> Fn(&'a str) -> Result<(T, &'a str), ParseError> {
    fn parse<'a>(&self, input: &'a str) -> Result<(T, &'a str), ParseError> {
        self(input)
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    IO,
    ExpectingCharacter(char),
    ExpectingEOF,

    ExpectingString(String),
    ExpectingPredicate,
    EndOfInput,
    ExpectingOneOfToParse,
    GenericError,
}

pub struct Character {
    character_to_match : char,
}

impl Character {
    pub fn new(character_to_match: char) -> impl Parser<char> {
        Self { character_to_match }
    }
}

pub fn character(character_to_match: char) -> impl Parser<char> {
    Character::new(character_to_match)
}

impl Parser<char> for Character {
    fn parse<'a>(&self, input: &'a str) -> Result<(char, &'a str), ParseError> {
        if input.starts_with(self.character_to_match) {
            Ok((self.character_to_match, &input[1..]))
        } else {
            Err(ParseError::ExpectingCharacter(self.character_to_match))
        }
    }
}

// pub struct Capture<T, P: Parser<T>> {
//     parser: P,
//     phantom: PhantomData<T>,
// }

// impl <T, P: Parser<T>> Parser<&'_ str> for Capture<T, P> {
//     fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, &'a str), ParseError> {
//         let start_len = input.len();
//         self.parser.parse(input).map(|(_, remaining)| {
//             let remaining_len = remaining.len();
//             // slice the input to return the portion of it that was matched by the parser
//             let result = &input[..(start_len - remaining_len)];
//             (result, remaining)
//         })
//     }
// }

// pub fn capture<'a, T, P: Parser<T>>(parser: P) -> impl Parser<&'_ str> {
//     |input: &str| {
//         let start_len = input.len();
//         parser.parse(input).map(|(_, remaining)| {
//             let remaining_len = remaining.len();
//             // slice the input to return the portion of it that was matched by the parser
//             let result = &input[..(start_len - remaining_len)];
//             (result, remaining)
//         })
//     }
//     // Capture { parser, phantom: PhantomData }
// }

pub fn many<T>(parser: impl Parser<T>) -> impl Parser<Vec<T>> {
    at_least(0, parser)
}


pub struct AtLeast<T, P> where P: Parser<T> + Sized {
    n: u8,
    parser: P,
    phantom: PhantomData<T>,
}

impl<T, P> AtLeast<T, P> where P: Parser<T> + Sized {
    pub fn new(n: u8, parser: P) -> Self {
        AtLeast { n, parser, phantom: PhantomData }
    }
}

pub fn at_least<T, P: Parser<T> + Sized>(n: u8, parser: P) -> AtLeast<T, P> {
    AtLeast::new(n, parser)
}

impl<T, P> Parser<Vec<T>> for AtLeast<T, P> where P: Parser<T> + Sized {
    fn parse<'a>(&self, input: &'a str) -> Result<(Vec<T>, &'a str), ParseError> {
        let mut result = vec![];
        let mut source = input;
        let mut count = self.n;
        while count > 0 {
            let attempt = self.parser.parse(source);
            match attempt {
                Ok((value, rest)) => {
                    result.push(value);
                    source = rest;
                }

                Err(e) => {
                    return Err(e);
                }
            }
            count -= 1;
        }
        loop {
            let attempt = self.parser.parse(source);
            match attempt {
                Ok((value, rest)) => {
                    result.push(value);
                    source = rest;
                }

                Err(_) => {
                    break;
                }
            }
        }
        Ok((result, source))
    }
}

pub struct Any<F> where F: Fn(char) -> bool + Sized {
    predicate: F,
}

impl<F> Any<F> where F: Fn(char) -> bool + Sized {
    pub fn new(predicate: F) -> Self {
        Any { predicate }
    }
}

pub fn any<F>(predicate: F) -> impl Parser<char> where F: Fn(char) -> bool + Sized {
    Any::new(predicate)
}

impl<F> Parser<char> for Any<F> where F: Fn(char) -> bool + Sized {
    fn parse<'a>(&self, input: &'a str) -> Result<(char, &'a str), ParseError> {
        let character = input.chars().next();
        match character {
            Some(c) => {
                if (self.predicate)(c) {
                    Ok((c, &input[1..]))
                } else {
                    Err(ParseError::ExpectingPredicate)
                }
            },

            None => {
                Err(ParseError::EndOfInput)
            }
        }
    }
}

pub struct Map<I, O, P, F> where I: 'static, P: Parser<I> + Sized, F: Fn(I) -> O + Sized {
    parser: P,
    map: F,
    phantom: PhantomData<I>,
}

impl<I, O, P, F> Map<I, O, P, F> where I: 'static, P: Parser<I> + Sized, F: Fn(I) -> O + Sized {
    pub fn new(parser: P, map: F) -> Self {
        Map { parser, map, phantom: PhantomData }
    }
}

pub fn map<I, O, P, F>(parser: P, map: F) -> impl Parser<O> where I: 'static, P: Parser<I> + Sized, F: Fn(I) -> O + Sized {
    Map::new(parser, map)
}

impl<I, O, P, F> Parser<O> for Map<I, O, P, F> where I: 'static, P: Parser<I> + Sized, F: Fn(I) -> O + Sized {
    fn parse<'a>(&self, input: &'a str) -> Result<(O, &'a str), ParseError> {
        let attempt = self.parser.parse(input);
        attempt.map(|(v, rest)|{ ((self.map)(v), rest)})
    }
}

pub struct OneOf<T, P> where T: 'static, P: Parser<T> + Sized {
    options: Vec<P>,
    phantom: PhantomData<T>,
}

impl<T, P> OneOf<T, P> where T: 'static, P: Parser<T> + Sized {
    pub fn new(options: Vec<P>) -> Self {
        Self { options, phantom: PhantomData }
    }
}

pub fn one_of<T, P>(options: Vec<P>) -> impl Parser<T> where T: 'static, P: Parser<T> + Sized {
    OneOf::new(options)
}

impl<T, P> Parser<T> for OneOf<T, P> where T: 'static, P: Parser<T> + Sized {
    fn parse<'a>(&self, input: &'a str) -> Result<(T, &'a str), ParseError> {
        for ref parser in &self.options {
            let attempt = parser.parse(input);
            if attempt.is_ok() {
                return attempt
            }
        }
        Err(ParseError::ExpectingOneOfToParse)
    }
}

pub struct Literal<'p>(&'p str);
impl <'p> Parser<()> for Literal<'p> {
    fn parse<'a>(&self, input: &'a str) -> Result<((), &'a str), ParseError> {
        if input.starts_with(self.0) {
            let len = self.0.len();
            let rem = &input[len..];
            Ok(((), rem))
        } else {
            Err(ParseError::ExpectingString(self.0.to_owned()))
        }
    }
}

pub fn literal(match_exactly: &str) -> Literal {
    Literal(match_exactly)
}

pub fn skip_spaces(input: &str) -> Result<((), &str), ParseError> {
    let byte_count = input.chars().take_while(|c| *c == ' ' || *c == '\t').count();
    Ok(((), &input[byte_count..]))
}


fn eof(input: &str) -> Result<((), &str), ParseError> {
    if input.is_empty() {
        Ok(((), input))
    } else {
        Err(ParseError::ExpectingEOF)
    }
}

pub struct Complete<T, P: Parser<T>>(P, PhantomData<T>);
impl <T, P: Parser<T>> Parser<T> for Complete<T, P> {
    fn parse<'a>(&self, input: &'a str) -> Result<(T, &'a str), ParseError> {
        let (res, rem) = self.0.parse(input)?;
        let (_, rem) = eof(rem)?;
        Ok((res, rem))
    }
}

pub fn complete<T>(parser: impl Parser<T>) -> impl Parser<T> {
    // move |input| {
    //     let (res, rem) = parser.parse(input)?;
    //     let (_, rem) = eof(rem)?;
    //     Ok((res, rem))
    // }
    Complete(parser, PhantomData)
}

#[macro_export]
macro_rules! parse_sequence {
    ($(let $name:ident = $parser:expr),+ => $finish:expr ) => {{
        |input: &str| {
            let rem: &_ = input;
            $(
                let ($name, rem) =$parser.parse(rem)?;
            )*
            let result = { $finish };
            Ok((result, rem))
        }
    }};
}


pub struct IgnoreLeadingWS<T, P: Parser<T>> {
    parser: P,
    _phantom: PhantomData<T>,
}

impl <T, P: Parser<T>> Parser<T> for IgnoreLeadingWS<T, P> {
    fn parse<'a>(&self, input: &'a str) -> Result<(T, &'a str), ParseError> {
        let (_, rem) = skip_spaces(input)?;
        let (value, rem) = self.parser.parse(input)?;
        let (_, rem) = skip_spaces(input)?;
        Ok((value, rem))
    }
}

pub fn ignore_leading_ws<T, P: Parser<T>>(parser: P) -> IgnoreLeadingWS<T, P> {
    IgnoreLeadingWS {
        parser,
        _phantom: PhantomData,
    }
}

#[macro_export]
macro_rules! parser_ignore_spaces {
    ($parser_name:ident $output_type:ty, $(let $name:ident = $parser:expr),+ => $finish:expr ) => {
        pub fn $parser_name<'a>(input: &'a str) -> Result<($output_type, &'a str), ParseError> {
            let rem: &'a str = input;
            $(
                let (_, rem) = $crate::combinator::skip_spaces(rem)?;
                let ($name, rem) = $parser.parse(rem)?;
            )*
            let (_, rem) = $crate::combinator::skip_spaces(rem)?;
            let result = { $finish };

            Ok((result, rem))
        }
    }
}

#[macro_export]
macro_rules! parse_sequence_ignore_spaces {
    ($(let $name:ident = $parser:expr),+ => $finish:expr ) => {
        move |input: &str| {
            let rem: &_ = input;
            $(
                let (_, rem) = $crate::combinator::skip_spaces(rem)?;
                let ($name, rem) =$parser.parse(rem)?;
            )*
            let (_, rem) = $crate::combinator::skip_spaces(rem)?;
            let result = { $finish };
            Ok((result, rem))
        }
    };
    ($input:expr => $rem:tt) => {
        let parser = parse_sequence_ignore_spaces!($rem);
        parser($input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_literal_string() {
        let (res, rem) = literal("foo").parse("foo").unwrap();
        assert_eq!("foo", res);
        assert!(rem.is_empty());
    }

    #[test]
    fn parse_sequence_ignore_spaces_using_macro() {

        let parser = parse_sequence_ignore_spaces!{
            let a = character('A'),
            let _foo = literal("foo"),
            let c = character('C')
            =>
            (a, c)
        };
        let (result, rem) = parser.parse(" \t A foo\t C  \t ").unwrap();
        assert_eq!(('A', 'C'), result);
        assert!(rem.is_empty());
    }

    #[test]
    fn parse_sequence_using_macro() {
        let parser = parse_sequence!{
            let a = character('A'),
            let b = character('b')
            =>
            (a, b)
        };
        let (result, rem) = parser.parse("Ab").expect("failed to parse");
        assert_eq!(('A', 'b'), result);
        assert!(rem.is_empty());
    }



    #[test]
    fn parse_a_character() {
        let input = "ABCD";
        let parser = character('A');

        let actual = parser.parse(input);

        let expected = Ok(('A', "BCD"));
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_many_a_characters() {
        let input = "AAABCD";
        let parser = many(character('A'));

        let actual = parser.parse(input);

        let expected = Ok((vec!['A', 'A', 'A'], "BCD"));
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_at_least_2_a_characters() {
        let input = "AAABCD";
        let parser = at_least(2, character('A'));

        let actual = parser.parse(input);

        let expected = Ok((vec!['A', 'A', 'A'], "BCD"));
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_any_character() {
        let input = "AAABCD";
        let parser = any(|c: char|{ c.is_alphabetic() });

        let actual = parser.parse(input);

        let expected = Ok(('A', "AABCD"));
        assert_eq!(actual, expected);
    }

    #[test]
    fn map_many_a_character_to_length() {
        let input = "AAABCD";
        let parser = map(many(character('A')), |cs|{cs.len()});

        let actual = parser.parse(input);

        let expected = Ok((3, "BCD"));
        assert_eq!(actual, expected);
    }

    #[test]
    fn  parse_one_of_b_or_a_characters() {
        let input = "AAABCD";
        let parser = one_of(vec![
            character('B'),
            character('A'),
        ]);

        let actual = parser.parse(input);

        let expected = Ok(('A', "AABCD"));
        assert_eq!(actual, expected);
    }
}
