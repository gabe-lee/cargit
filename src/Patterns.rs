pub struct PatternMatch<T> {
    pub index: usize,
    pub length: usize,
    pub slice: T,
}

impl<T> PatternMatch<T> {
    pub fn start(&self) -> usize {
        return self.index;
    }

    pub fn end(&self) -> usize {
        return self.index+self.length;
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        return self.index..self.index+self.length;
    }
}

pub trait PatternMatcher<'a, P> {
    fn find_first_from(&'a self, pattern: &P, byte_offset: usize) -> Option<PatternMatch<&'a Self>>;

    #[inline(always)]
    fn find_first(&'a self, pattern: &P) -> Option<PatternMatch<&'a Self>>{
        return self.find_first_from(pattern, 0);
    }

    fn find_every_from(&'a self, pattern: &P, byte_offset: usize) -> Option<Vec<PatternMatch<&'a Self>>> {
        let mut total_offset: usize = byte_offset;
        let mut matches = Vec::new();
        loop {
            match self.find_first_from(pattern, total_offset) {
                Some(found_match) => {
                    total_offset = found_match.end();
                    matches.push(found_match)
                },
                None => break
            }
        }
        if matches.is_empty() {
            return None;
        }
        return Some(matches);
    }

    #[inline(always)]
    fn find_every(&'a self, pattern: &P) -> Option<Vec<PatternMatch<&'a Self>>> {
        return self.find_every_from(pattern, 0);
    }

    fn find_any_from<IIP: IntoIterator<Item = P>>(&'a self, patterns: IIP, byte_offset: usize) -> Option<PatternMatch<&'a Self>> {
        let mut matches = Vec::new();
        for pattern in patterns.into_iter() {
            if let Some(found_match) = self.find_first_from(&pattern, byte_offset) {
                matches.push(found_match);
            }
        }
        let mut earliest_match: Option<PatternMatch<&'a Self>> = None;
        let mut earliest_index: usize = usize::MAX;
        for current_match in matches {
            if earliest_match.is_none() || current_match.index < earliest_index {
                earliest_index = current_match.index;
                earliest_match = Some(current_match);
            }
        }
        return earliest_match;
    }

    #[inline(always)]
    fn find_any<IIP: IntoIterator<Item = P>>(&'a self, patterns: IIP) -> Option<PatternMatch<&'a Self>> {
        self.find_any_from(patterns, 0)
    }

    fn find_all_from<IIP: IntoIterator<Item = P>>(&'a self, patterns: IIP, byte_offset: usize) -> Option<Vec<PatternMatch<&'a Self>>> {
        let mut matches = Vec::new();
        for pattern in patterns.into_iter() {
            if let Some(mut found_matches) = self.find_every_from(&pattern, byte_offset) {
                matches.append(&mut found_matches);
            }
        }
        if matches.is_empty() {
            return None;
        }
        return Some(matches);
    }

    #[inline(always)]
    fn find_all<IIP: IntoIterator<Item = P>>(&'a self, patterns: IIP) -> Option<Vec<PatternMatch<&'a Self>>> {
        return self.find_all_from(patterns, 0)
    }
}

impl<'a, P> PatternMatcher<'a, P> for str
where P: AsRef<str> {
    fn find_first_from(&'a self, pattern: &P, byte_offset: usize) -> Option<PatternMatch<&'a str>> {
        let pattern_str = pattern.as_ref();
        if let Some(index) = self[byte_offset..].find(pattern_str) {
            let byte_len = pattern_str.len();
            let slice = &self[byte_offset + index..byte_offset + index + byte_len];
            Some(PatternMatch { index: byte_offset + index, length: byte_len, slice })
        } else {
            None
        }
    }
}


impl<'a, P, T> PatternMatcher<'a, P> for [T]
where P: AsRef<[T]>,
T: PartialEq {
    fn find_first_from(&'a self, pattern: &P, byte_offset: usize) -> Option<PatternMatch<&'a Self>> {
        let pattern_slice = pattern.as_ref();
        let pattern_len = pattern_slice.len();
        let offset_slice = &self[byte_offset..];
        for compare_start in 0..offset_slice.len() {
            let compare_end = compare_start + pattern_len;
            if compare_end > offset_slice.len() {
                return None;
            }
            let compare_slice = &offset_slice[compare_start..compare_end];
            if compare_slice == pattern_slice {
                let true_index = compare_start+byte_offset;
                return Some(PatternMatch { index: true_index, length: pattern_len, slice: &self[true_index..true_index+pattern_len] })
            }
        }
        return None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_match_start() {
        let pm = PatternMatch { index: 2, length: 3, slice: "hello" };
        assert_eq!(pm.start(), 2);
    }

    #[test]
    fn test_pattern_match_end() {
        let pm = PatternMatch { index: 2, length: 3, slice: "hello" };
        assert_eq!(pm.end(), 5);
    }

    #[test]
    fn test_pattern_match_range() {
        let pm = PatternMatch { index: 2, length: 3, slice: "hello" };
        assert_eq!(pm.range(), 2..5);
    }

    #[test]
    fn test_find_first() {
        let s = "hello world";
        let pm = s.find_first(&"world").unwrap();
        assert_eq!(pm.start(), 6);
        assert_eq!(pm.end(), 11);
        assert_eq!(pm.slice, "world");
    }

    #[test]
    fn test_find_every() {
        let s = "hello world";
        let pms = s.find_every(&"l").unwrap();
        assert_eq!(pms.len(), 3);
        assert_eq!(pms[0].start(), 2);
        assert_eq!(pms[0].end(), 3);
        assert_eq!(pms[1].start(), 3);
        assert_eq!(pms[1].end(), 4);
        assert_eq!(pms[2].start(), 9);
        assert_eq!(pms[2].end(), 10);
    }

    #[test]
    fn test_find_any() {
        let s = "hello world";
        let pm = s.find_any(&["world", "foo", "bar"]).unwrap();
        assert_eq!(pm.start(), 6);
        assert_eq!(pm.end(), 11);
        assert_eq!(pm.slice, "world");
    }

    #[test]
    fn test_find_all() {
        let s = "hello world";
        let pms = s.find_all(&["l", "o"]).unwrap();
        assert_eq!(pms.len(), 5);
        assert_eq!(pms[0].start(), 2);
        assert_eq!(pms[0].end(), 3);
        assert_eq!(pms[1].start(), 3);
        assert_eq!(pms[1].end(), 4);
        assert_eq!(pms[2].start(), 9);
        assert_eq!(pms[2].end(), 10);
        assert_eq!(pms[3].start(), 4);
        assert_eq!(pms[3].end(), 5);
        assert_eq!(pms[4].start(), 7);
        assert_eq!(pms[4].end(), 8);
    }
}
