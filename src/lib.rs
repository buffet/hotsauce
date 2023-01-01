//! Regex search over iterators of bypes.
//! Why can't Rust users stop hardcoding `&str` everywhere?
#![warn(missing_docs, unreachable_pub)]

use std::{iter::Enumerate, ops::Range, convert::TryFrom};

use regex_automata::{dense, DenseDFA, DFA};

pub use regex_automata::Error;

type Automata = DenseDFA<Vec<usize>, usize>;

/// A regular expression.
#[derive(Debug, Clone)]
pub struct Regex {
    fw: Automata,
    bw: Automata,
}

/// An iterator over the (non-overlapping) matches.
#[derive(Debug)]
pub struct Matches<'r, Haystack: Iterator<Item = u8>> {
    haystack: Enumerate<Haystack>,
    dfa: &'r Automata,
}

impl Regex {
    /// Build a new regex from the given string.
    /// This uses `regex-syntax`, see that for more documentation.
    pub fn new(re: &str) -> Result<Regex, Error> {
        Ok(Regex {
            fw: dense::Builder::new().anchored(true).build(re)?,
            bw: dense::Builder::new()
                .anchored(true)
                .reverse(true)
                .build(re)?,
        })
    }

    /// Returns an iterator over the matches.
    ///
    /// ```rust
    /// use hotsauce::Regex;
    ///
    /// let regex = Regex::new("hey");
    /// let match = regex.matches("abc hey".bytes()).next();
    /// assert_eq!(Some(4..7), match);
    /// ```
    pub fn matches<Haystack: Iterator<Item = u8>>(&self, haystack: Haystack) -> Matches<Haystack> {
        Matches::new(&self.fw, haystack)
    }

    /// Returns an iterator over the matches, searching backwards.
    /// The iterator needs to go backwards.
    /// The matches returned will be indeces into the iterator, see the example.
    ///
    /// ```rust
    /// use hotsauce::Regex;
    ///
    /// let regex = Regex::new("hey");
    /// let match = regex.rmatches("hey abc".bytes().rev()).next();
    /// assert_eq!(Some(4..7), match);
    /// ```
    pub fn rmatches<Haystack: Iterator<Item = u8>>(&self, haystack: Haystack) -> Matches<Haystack> {
        Matches::new(&self.bw, haystack)
    }
}

impl TryFrom<&str> for Regex {
    type Error = Error;

    fn try_from(str: &str) -> Result<Self, Self::Error> {
        Regex::new(str)
    }
}

impl<Haystack: Iterator<Item = u8>> Matches<'_, Haystack> {
    fn new(dfa: &Automata, haystack: Haystack) -> Matches<Haystack> {
        Matches {
            haystack: haystack.enumerate(),
            dfa,
        }
    }
}

impl<Haystack: Iterator<Item = u8>> Iterator for Matches<'_, Haystack> {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let dfa = self.dfa;
        let start_state = dfa.start_state();

        if dfa.is_dead_state(start_state) {
            return None;
        }

        let mut states = vec![];

        while let Some((i, b)) = self.haystack.next() {
            let end = dfa.is_match_state(start_state).then_some(i);
            states.push((i, end, start_state));

            for (start, end, state) in &mut states {
                *state = unsafe { dfa.next_state_unchecked(*state, b) };
                if dfa.is_dead_state(*state) {
                    match end {
                        Some(end) => return Some(*start..*end + 1),
                        None => continue,
                    }
                }
                if dfa.is_match_state(*state) {
                    *end = Some(i);
                }
            }

            states.retain(|&(_, _, state)| !dfa.is_dead_state(state));
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
