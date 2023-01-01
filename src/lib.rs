//! Regex search over iterators of bypes.
//! Why can't Rust users stop hardcoding `&str` everywhere?
#![warn(missing_docs, unreachable_pub)]

use std::{convert::TryFrom, iter::Enumerate, ops::Range};

use regex_automata::{dense, DenseDFA, DFA};

pub use regex_automata::Error;

type Automata = DenseDFA<Vec<usize>, usize>;

/// A regular expression.
#[derive(Debug, Clone)]
pub struct Regex {
    fw: Automata,
    bw: Automata,
}

/// A builder for a regex from a string.
/// This allows several configuration options, such as unicode support and case sensitivity.
/// For basically all of these options, see regex_automata::dense::Builder.
///
/// ```rust
/// use hotsauce::RegexBuilder;
///
/// let regex = RegexBuilder::new()
///     .case_insensitive(true)
///     .build("hello");
/// let match = regex.matches("HeLlO".bytes()).next();
/// assert_eq!(Some(0..5), match);
/// ````
#[derive(Debug, Clone)]
pub struct RegexBuilder(dense::Builder);

/// An iterator over the (non-overlapping) matches.
#[derive(Debug)]
pub struct Matches<'r, Haystack: Iterator<Item = u8>> {
    haystack: Enumerate<Haystack>,
    dfa: &'r Automata,
}

impl Regex {
    /// Build a new regex from the given string with default settings (see RegexBuilder).
    /// This uses `regex-syntax`, see that for more documentation.
    pub fn new(re: &str) -> Result<Regex, Error> {
        RegexBuilder::new().build(re)
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

impl RegexBuilder {
    /// Create a new Regex builder.
    pub fn new() -> RegexBuilder {
        let mut builder = dense::Builder::new();
        builder.anchored(true);
        Self(builder)
    }

    /// Build the regex with the given expression.
    pub fn build(&self, re: &str) -> Result<Regex, Error> {
        Ok(Regex {
            bw: self.0.clone().reverse(true).build(re)?,
            fw: self.0.build(re)?,
        })
    }

    /// Enable case insensitivity.
    /// This is disabled by default.
    pub fn case_insensitive(&mut self, yes: bool) -> &mut RegexBuilder {
        self.0.case_insensitive(yes);
        self
    }

    /// Allow or disallow the use of whitespace and comments in regex.
    /// This is disabled by default.
    pub fn verbose(&mut self, yes: bool) -> &mut RegexBuilder {
        self.0.ignore_whitespace(yes);
        self
    }

    /// Set whether dot should match new line characters.
    /// Disabled by default.
    pub fn dot_matches_new_line(&mut self, yes: bool) -> &mut RegexBuilder {
        self.0.dot_matches_new_line(yes);
        self
    }

    /// Enable or disable "swap greed".
    /// Disabled by default.
    pub fn swap_greed(&mut self, yes: bool) -> &mut RegexBuilder {
        self.0.swap_greed(yes);
        self
    }

    /// Enable or disable unicode.
    /// Enabled by default.
    pub fn unicode(&mut self, yes: bool) -> &mut RegexBuilder {
        self.0.unicode(yes);
        self
    }

    /// Allows the construction of &mut Regex that match invalid UTF-8.
    pub fn allow_invalid_utf8(&mut self, yes: bool) -> &mut RegexBuilder {
        self.0.allow_invalid_utf8(yes);
        self
    }

    /// Set the nest limit used for the parser.
    pub fn nest_limit(&mut self, limit: u32) -> &mut RegexBuilder {
        self.0.nest_limit(limit);
        self
    }

    /// Minimize the DFA to be as small as possible.
    /// Disabled by default.
    pub fn minimize(&mut self, yes: bool) -> &mut RegexBuilder {
        self.0.minimize(yes);
        self
    }

    /// Premultiply the transition table.
    /// Enabled by default.
    pub fn premultiply(&mut self, yes: bool) -> &mut RegexBuilder {
        self.0.premultiply(yes);
        self
    }

    /// Shrink the size of the DFA’s alphabet by mapping bytes to their equivalence classes.
    /// Enabled by default.
    pub fn byte_classes(&mut self, yes: bool) -> &mut RegexBuilder {
        self.0.byte_classes(yes);
        self
    }
}

impl Default for RegexBuilder {
    fn default() -> Self {
        Self::new()
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
