#![feature(array_zip)]

use std::io;
use std::io::{BufRead, stdin, Write};
use std::sync::{Arc};
use dashmap::DashMap;
use rayon::prelude::*;

fn main() {
    let words: Vec<[u8; 5]> = include_str!("words.txt").lines().map(
        |line| {
            assert_eq!(line.len(), 5);
            let mut array = [b'\0'; 5];
            line.chars().into_iter().enumerate().for_each(|(i, c)| array[i] = c as u8);
            array
        }
    ).collect();

    let mut config = WordleConfig::default();
    let stdin = stdin();
    let lock = stdin.lock();
    let mut input = lock.lines();
    loop {
        println!("Enter: input/calc");
        match input.next().unwrap().unwrap().as_str() {
            "input" => {
                print!("Enter positions (? for none): ");
                io::stdout().flush();
                let greens: [Option<u8>; 5] = input.next().unwrap().unwrap().chars().map(|c| if c == '?' { None } else { Some(c as u8) }).collect::<Vec<_>>().as_slice().try_into().unwrap();

                print!("Enter frequencies (list all green + yellow): ");
                io::stdout().flush();
                let mut freqs = [0u8; 26];
                input.next().unwrap().unwrap().chars().for_each(|c| freqs[c as usize - 'a' as usize] += 1);

                print!("Enter exact (all black letters): ");
                io::stdout().flush();
                let mut exacts = [false; 26];
                input.next().unwrap().unwrap().chars().for_each(|c| exacts[c as usize - 'a' as usize] = true);

                let word_config = WordleConfig {
                    positions: greens,
                    freqs_min: freqs,
                    freqs_exact: exacts
                };
                config = config.merge(word_config);
                println!("{:?}", config);
            },
            "calc" => {
                // print!("Enter depth: ");
                // io::stdout().flush();
                // let depth: usize = input.next().unwrap().unwrap().parse().unwrap();

                // let mut cache = HashMap::new();
                // let (best_word, best_score) = optimize(config, &words, &mut cache, f64::INFINITY, depth, true);
                // println!("Guess: {} ({})", best_word.iter().map(|c| *c as char).collect::<String>(), best_score);

                let cache = Arc::new(DashMap::new());
                let (best_word, best_score) = optimize_new(config, &words, cache);
                println!("Guess: {} ({})", best_word.iter().map(|c| *c as char).collect::<String>(), best_score);
            },
            _ => {
                continue
            }
        }
    }



}

#[inline(always)]
fn optimize_new(config: WordleConfig, words: &Vec<[u8; 5]>, cache: Arc<DashMap<WordleConfig, f64>>) -> ([u8; 5], f64) {
    let subwords: Vec<_> = words.clone().into_iter().filter(|&w| config.matches_word(w)).collect();
    let mut guesses = subwords.par_iter().enumerate().map(|(i, &guess)| {
        println!("{}", i);
        let (count, sum) = subwords.iter().map(|&correct| {
            let word_config = WordleConfig::from_guess_and_correct(guess, correct);
            let config_new = config.merge(word_config);

            if let Some(v) = cache.get(&config_new) {
                *v.value()
            } else {
                //Find words leftover
                let v = subwords.iter().filter(|&&word| config_new.matches_word(word)).count() as f64;
                cache.insert(config_new, v);
                v
            }

        }).fold((0f64, 0f64), |(count, sum), next| (count + 1f64, sum + next));
        (guess, sum / count)
    }).collect::<Vec<([u8; 5], f64)>>();
    guesses.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    guesses[0]
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct WordleConfig {
    positions: [Option<u8>; 5],
    freqs_min: [u8; 26],
    freqs_exact: [bool; 26]
}

impl Default for WordleConfig {
    fn default() -> Self {
        WordleConfig {
            positions: [None; 5],
            freqs_min: [0; 26],
            freqs_exact: [false; 26]
        }
    }
}

impl WordleConfig {
    #[inline(always)]
    fn from_guess_and_correct(guess: [u8; 5], correct: [u8; 5]) -> Self {
        WordleConfig {
            positions: guess.zip(correct).map(|(gc, cc)| if gc == cc { Some(gc) } else { None }),
            freqs_min: word_freqs(&guess).zip(word_freqs(&correct)).map(|(g, c)| g.min(c)),
            freqs_exact: word_freqs(&guess).zip(word_freqs(&correct)).map(|(g, c)| g > c),
        }
    }
    #[inline(always)]
    fn merge(self: Self, other: Self) -> Self {
        Self {
            positions: self.positions.zip(other.positions).map(|(a, b)| a.or(b)),
            freqs_min: self.freqs_min.zip(other.freqs_min).map(|(a, b)| a.max(b)),
            freqs_exact: self.freqs_exact.zip(other.freqs_exact).map(|(a, b)| a || b),
        }
    }

    #[inline(always)]
    fn matches_word(&self, word: [u8; 5]) -> bool {
        //Check positions
        if !self.positions.zip(word).into_iter().all(|(mc, c)| mc.is_none() || mc.unwrap() == c) { return false }

        //Check freqs
        return word_freqs(&word).zip(self.freqs_min.zip(self.freqs_exact)).into_iter().all(|(freq, (freq_min, freq_exact))| freq_min <= freq && (!freq_exact || freq_min == freq))
        // return word_freqs(&word).zip(self.freqs_min).into_iter().all(|(freq, freq_min)| freq_min <= freq)
    }

    #[inline(always)]
    fn is_finished(&self) -> bool {
        self.positions.iter().all(|c| c.is_some())
    }
}

#[inline(always)]
fn word_freqs(word: &[u8; 5]) -> [u8; 26] {
    let mut in_word = [0u8; 26];
    word.iter().for_each(|letter| in_word[(*letter as usize) - ('a' as usize)] += 1);
    in_word
}