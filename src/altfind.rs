pub struct StringMatch<'a> {
    index: usize,
    length: usize,
    value: &'a str
}

pub trait StringPattern {
    fn find_in<'a>(self, string: &'a str) -> Option<StringMatch<'a>>;
}

impl StringPattern for char {
    fn find_in<'a>(self, string: &'a str) -> Option<StringMatch<'a>> {
        !todo() // Find single char match and return index, byte length, and string slice of match location
    }
}

impl StringPattern for &str {
    fn find_in<'a>(self, string: &'a str) -> Option<StringMatch<'a>> {
        !todo() // Find string match and return index, byte length, and string slice of match location
    }
}

impl StringPattern for Vec<char> {
    fn find_in<'a>(self, string: &'a str) -> Option<StringMatch<'a>> {
        !todo() // Find any char match and return index, byte length, and string slice of match location
    }
}

impl StringPattern for Vec<&str> {
    fn find_in<'a>(self, string: &'a str) -> Option<StringMatch<'a>> {
        !todo() // Find any string match and return index, byte length, and string slice of match location
    }
}

pub struct StringQulaifierMatch {
    index: usize,
    length: usize,
}

impl<F> StringPattern for F 
where F : Fn(&str) -> Option<StringQulaifierMatch> {
    fn find_in<'a>(self, string: &'a str) -> Option<StringMatch<'a>> {
        !todo() // Find match by iteratively increasing char index and passing &string[index..] to closure looking for Some(StringQulaifierMatch)
    }
}