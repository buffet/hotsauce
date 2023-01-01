//! Regex search over iterators of bypes.
//! Why can't Rust users stop hardcoding `&str` everywhere?
#![warn(missing_docs, unreachable_pub)]

use std::{iter::Enumerate, ops::Range};

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
    haystack: Enumerate<Haystack>,
    regex: &'r Regex,
}

impl Regex {
    /// Build a new regex.
    pub fn new(re: &str) -> Result<Regex, Error> {
        Ok(Regex {
            fw: dense::Builder::new().anchored(true).build(re)?,
        })
    }

    /// Returns an iterator over the matches.
    pub fn matches<Haystack: Iterator<Item = u8>>(&self, haystack: Haystack) -> Matches<Haystack> {
        Matches::new(self, haystack)
    }
}

impl<Haystack: Iterator<Item = u8>> Matches<'_, Haystack> {
    fn new(regex: &Regex, haystack: Haystack) -> Matches<Haystack> {
        Matches {
            haystack: haystack.enumerate(),
            regex,
        }
    }
}

impl<Haystack: Iterator<Item = u8>> Iterator for Matches<'_, Haystack> {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let re = &self.regex.fw;
        let start_state = re.start_state();

        if re.is_dead_state(start_state) {
            return None;
        }

        let mut states = vec![];

        while let Some((i, b)) = self.haystack.next() {
            let end = re.is_match_state(start_state).then_some(i);
            states.push((i, end, start_state));

            for (start, end, state) in &mut states {
                *state = unsafe { re.next_state_unchecked(*state, b) };
                if re.is_dead_state(*state) {
                    match end {
                        Some(end) => return Some(*start..*end + 1),
                        None => continue,
                    }
                }
                if re.is_match_state(*state) {
                    *end = Some(i);
                }
            }

            states.retain(|&(_, _, state)| !re.is_dead_state(state));
        }

        for (start, end, _) in states.iter().cloned() {
            match end {
                Some(end) => return Some(start..end + 1),
                None => {}
            }
        }

        None
    }
}
