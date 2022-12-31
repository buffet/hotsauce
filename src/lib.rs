//! Regex search over iterators of bypes.
//! Why can't Rust users stop hardcoding `&str` everywhere?
#![warn(missing_docs, unreachable_pub)]

use std::{iter::Enumerate, ops::Range};

use itertools::{Itertools, MultiPeek};
use regex_automata::{dense, DenseDFA, DFA};

pub use regex_automata::Error;

/// A regular expression.
#[derive(Debug, Clone)]
pub struct Regex {
    fw: DenseDFA<Vec<usize>, usize>,
}

/// An iterator over the (non-overlapping) matches.
#[derive(Debug)]
pub struct Matches<'r, Haystack: Iterator<Item = u8>> {
    haystack: MultiPeek<Enumerate<Haystack>>,
    regex: &'r mut Regex,
}

impl Regex {
    /// Build a new regex.
    pub fn new(re: &str) -> Result<Regex, Error> {
        Ok(Regex {
            fw: dense::Builder::new().anchored(true).build(re)?,
        })
    }

    /// Returns an iterator over the matches.
    pub fn matches<Haystack: Iterator<Item = u8>>(
        &mut self,
        haystack: Haystack,
    ) -> Matches<Haystack> {
        Matches::new(self, haystack)
    }
}

impl<Haystack: Iterator<Item = u8>> Matches<'_, Haystack> {
    fn new(regex: &mut Regex, haystack: Haystack) -> Matches<Haystack> {
        Matches {
            haystack: haystack.enumerate().multipeek(),
            regex,
        }
    }

    /// Tries to match at start of haystack, without advancing it.
    fn match_at_start(&mut self) -> Option<usize> {
        let re = &self.regex.fw;

        let mut state = re.start_state();
        if re.is_dead_state(state) {
            return None;
        }

        let mut end = re.is_match_state(state).then_some(0);

        while let Some((i, b)) = self.haystack.peek().cloned() {
            state = unsafe { re.next_state_unchecked(state, b) };
            if re.is_dead_state(state) {
                return end;
            }
            if re.is_match_state(state) {
                end = Some(i + 1)
            }
        }

        end
    }
}

impl<Haystack: Iterator<Item = u8>> Iterator for Matches<'_, Haystack> {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let start = self.haystack.peek()?.0;
            self.haystack.reset_peek();

            match self.match_at_start() {
                Some(end) => {
                    for _ in 0..end - start {
                        self.haystack.next();
                    }
                    return Some(start..end);
                }
                None => {
                    self.haystack.next();
                }
            }
        }
    }
}
