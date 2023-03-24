pub struct StringMatch<'a> {
    index: usize,
    length: usize,
    value: &'a str
}

pub struct StringQulaifierMatch {
    index: usize,
    length: usize,
}

pub enum StringPattern {
    Char(char),
    String(&str),
    CharList(Vec<char>),
    StringList(Vec<&str>),
    Qualifier(Box<dyn Fn(&str) -> Option<StringQulaifierMatch>>)
}

pub trait PatternMatching {
    pub fn match_pattern<'a>(&'a self, pattern: StringPattern) -> StringMatch<'a>;
}

impl PatternMatching for &str {
    fn match_pattern<'a>(&'a self, pattern: StringPattern) -> StringMatch<'a> {
        
    }
}