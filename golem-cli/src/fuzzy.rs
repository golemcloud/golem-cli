// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use colored::Colorize;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use golem_wasm_rpc_stubgen::log::LogColorize;
use itertools::Itertools;
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::Write;

pub enum FuzzyMatchResult<'a> {
    Found { option: &'a str, exact_match: bool },
    Ambiguous { highlighted_options: Vec<String> },
    NotFound,
}

pub struct FuzzySearch<'a> {
    options: HashSet<&'a str>,
    matcher: SkimMatcherV2,
}

impl<'a> FuzzySearch<'a> {
    pub fn new<I: Iterator<Item = &'a str>>(options: I) -> Self {
        let options_set = HashSet::from_iter(options);
        Self {
            options: options_set,
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn find(&self, pattern: &str) -> FuzzyMatchResult<'a> {
        // Exact matches
        if let Some(option) = self.options.get(pattern) {
            return FuzzyMatchResult::Found {
                option: *option,
                exact_match: true,
            };
        }

        // Contains matches
        let contains_matches = self
            .options
            .iter()
            .filter(|&option| option.contains(pattern))
            .collect::<Vec<_>>();

        if contains_matches.len() == 1 {
            return FuzzyMatchResult::Found {
                option: *contains_matches[0],
                exact_match: false,
            };
        }

        // Fuzzy matches
        let fuzzy_matches = self
            .options
            .iter()
            .filter_map(|option| {
                self.matcher
                    .fuzzy_indices(option, pattern)
                    .map(|(score, indices)| (score, indices, option))
            })
            .sorted_by(|(score_a, _, _), (score_b, _, _)| Ord::cmp(score_b, score_a))
            .collect::<Vec<_>>();

        match fuzzy_matches.len() {
            0 => FuzzyMatchResult::NotFound,
            1 => FuzzyMatchResult::Found {
                option: fuzzy_matches[0].2,
                exact_match: false,
            },
            _ => FuzzyMatchResult::Ambiguous {
                highlighted_options: fuzzy_matches
                    .into_iter()
                    .map(|(_, indices, option)| {
                        let indices = HashSet::<usize>::from_iter(indices.into_iter());
                        let mut highlighted_option = String::with_capacity(option.len() * 2);
                        for (idx, char) in option.chars().enumerate() {
                            if indices.contains(&idx) {
                                highlighted_option
                                    .write_fmt(format_args!(
                                        "{}",
                                        char.to_string().green().underline()
                                    ))
                                    .unwrap();
                            } else {
                                highlighted_option.write_char(char).unwrap()
                            }
                        }
                        highlighted_option
                    })
                    .collect(),
            },
        }
    }
}
