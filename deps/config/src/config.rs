use log::LevelFilter;

use crate::parser::Parser;

#[derive(Debug, Clone, Eq)]
pub struct Config {
    pub log: log::LevelFilter,
    pub com1: bool,
    pub cmdline: &'static str,
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.log == other.log && self.com1 == other.com1
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log: default_log(),
            com1: default_com1(),
            cmdline: "",
        }
    }
}

impl Config {
    pub fn from_cmdline(cmdline: &'static [u8]) -> Config {
        let Ok(s) = core::str::from_utf8(cmdline) else {
            return Self::default();
        };

        Self::from_cmdline_str(s)
    }

    pub fn from_cmdline_str(cmdline: &'static str) -> Config {
        let mut cfg = Config {
            cmdline,
            ..Default::default()
        };

        let parser = Parser::new(cmdline);

        for item in parser {
            let Ok(item) = item else {
                // todo: report errors somehow
                continue;
            };

            item.apply(&mut cfg);
        }

        cfg
    }
}

fn default_log() -> LevelFilter {
    LevelFilter::Info
}

fn default_com1() -> bool {
    true
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("log=invalid", Default::default())]
    #[case("log=warn", Config { log: LevelFilter::Warn, ..Default::default() })]
    #[case("log=invalid, com1=false", Config { com1: false, ..Default::default() })]
    #[case("log=warn com1=false", Config { log: LevelFilter::Warn, com1: false, ..Default::default() })]
    fn from_cmdline(#[case] cmdline: &str, #[case] expected: Config) {
        let cmdline = Config::from_cmdline_str(cmdline);
        assert_eq!(cmdline, expected);
    }
}
