use core::str::SplitWhitespace;

use log::LevelFilter;

use crate::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedValue {
    Log(LevelFilter),
    Com1(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedValueError {
    Log,
    Com1,
    IllFormedPair,
    UnknownProperty,
}

impl ParsedValue {
    pub fn apply(self, config: &mut Config) {
        match self {
            ParsedValue::Log(log) => config.log = log,
            ParsedValue::Com1(com1) => config.com1 = com1,
        }
    }
}

pub struct Parser<'a> {
    cmdline: SplitWhitespace<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(cmdline: &'a str) -> Self {
        Self {
            cmdline: cmdline.split_whitespace(),
        }
    }
}

impl Iterator for Parser<'_> {
    type Item = Result<ParsedValue, ParsedValueError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.cmdline.next().map(parse_pair)
    }
}

fn parse_pair(kv: &str) -> Result<ParsedValue, ParsedValueError> {
    let (key, value) = match kv.split_once('=') {
        Some(kv) => kv,
        None => return Err(ParsedValueError::IllFormedPair),
    };

    let subparser = match key {
        "log" => parse_log,
        "com1" => parse_com1,
        _ => return Err(ParsedValueError::UnknownProperty),
    };

    subparser(value)
}

fn parse_log(value: &str) -> Result<ParsedValue, ParsedValueError> {
    let log = match value {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => return Err(ParsedValueError::Log),
    };

    Ok(ParsedValue::Log(log))
}

fn parse_com1(value: &str) -> Result<ParsedValue, ParsedValueError> {
    parse_bool(value)
        .map(ParsedValue::Com1)
        .ok_or(ParsedValueError::Com1)
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[test]
    fn illformed_pair() {
        let kv = "aaa";
        assert_eq!(parse_pair(kv), Err(ParsedValueError::IllFormedPair));
    }

    #[test]
    fn unknown_property() {
        let kv = "xxx=zzz";
        assert_eq!(parse_pair(kv), Err(ParsedValueError::UnknownProperty));
    }

    #[rstest]
    #[case("", 0)]
    #[case("a=a", 1)]
    #[case("a=a b=b c=c", 3)]
    fn items_count(#[case] cmdline: &str, #[case] expected_count: usize) {
        let parser = Parser::new(cmdline);
        let count = parser.count();

        assert_eq!(count, expected_count);
    }

    #[rstest]
    #[case("log=invalid", Err(ParsedValueError::Log))]
    #[case("log=off", Ok(ParsedValue::Log(LevelFilter::Off)))]
    #[case("log=error", Ok(ParsedValue::Log(LevelFilter::Error)))]
    #[case("log=warn", Ok(ParsedValue::Log(LevelFilter::Warn)))]
    #[case("log=info", Ok(ParsedValue::Log(LevelFilter::Info)))]
    #[case("log=debug", Ok(ParsedValue::Log(LevelFilter::Debug)))]
    #[case("log=trace", Ok(ParsedValue::Log(LevelFilter::Trace)))]
    fn log(#[case] kv: &str, #[case] result: Result<ParsedValue, ParsedValueError>) {
        assert_eq!(parse_pair(kv), result);
    }

    #[rstest]
    #[case("com1=invalid", Err(ParsedValueError::Com1))]
    #[case("com1=1", Ok(ParsedValue::Com1(true)))]
    #[case("com1=0", Ok(ParsedValue::Com1(false)))]
    #[case("com1=true", Ok(ParsedValue::Com1(true)))]
    #[case("com1=false", Ok(ParsedValue::Com1(false)))]
    fn com1(#[case] kv: &str, #[case] result: Result<ParsedValue, ParsedValueError>) {
        assert_eq!(parse_pair(kv), result);
    }
}
