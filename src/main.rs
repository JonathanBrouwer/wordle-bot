#![feature(array_zip)]

mod wordle_config;

use std::cmp::min;
use std::io;
use std::io::{BufRead, stdin, Write};
use std::sync::{Mutex};
use dashmap::DashMap;
use rayon::prelude::*;
use crate::wordle_config::{ComplexWordleConfig, num, WordleConfig};

fn main() {
    let words: Vec<[u8; 5]> = include_str!("words.txt").lines().map(
        |line| {
            assert_eq!(line.len(), 5);
            let mut array = [b'\0'; 5];
            line.chars().into_iter().enumerate().for_each(|(i, c)| array[i] = c as u8);
            array
        }
    ).collect();

    let mut config = ComplexWordleConfig::default();
    let stdin = stdin();
    let lock = stdin.lock();
    let mut input = lock.lines();
    loop {
        println!("Enter: input/calc");
        match input.next().unwrap().unwrap().as_str() {
            "input" => {
                print!("Enter letters/gyb: ");
                io::stdout().flush().unwrap();
                let chars: Vec<char> = input.next().unwrap().unwrap().chars().collect();

                let guess: [char; 5] = chars[0..5].try_into().unwrap();
                let guess: [u8; 5] = guess.map(|c| c as u8);
                let colors: [char; 5] = chars[6..11].try_into().unwrap();

                let mut positions = [[true; 26]; 5];
                for i in 0..5 {
                    //Char is correct
                    if colors[i] == 'g' {
                        for j in 0..26 {
                            positions[i][j] = false;
                        }
                        positions[i][num(guess[i]) as usize] = true;
                    }
                    //Char is incorrect
                    else {
                        positions[i][num(guess[i]) as usize] = false;
                    }
                }

                let mut freqs_min = [0; 26];
                let mut freqs_exact = [false; 26];
                guess.zip(colors).iter().for_each(|(g, c)| {
                   if *c == 'b' {
                       freqs_exact[num(*g) as usize] = true;
                   } else {
                       freqs_min[num(*g) as usize] += 1;
                   }
                });

                let word_config = ComplexWordleConfig { positions, freqs_min,freqs_exact };

                config = config.merge(word_config);
                println!("{:?}", config);
            },
            "calc" => {
                let words = optimize_new(config, &words);
                println!("Best guesses: ({} possibilities)", words.len());
                for i in 0..min(10, words.len()) {
                    println!("#{i}: {} ({})", words[i].0.map(|c| c as char).iter().collect::<String>(), words[i].1.round());
                }
            },
            _ => {
                continue
            }
        }
    }
}

#[inline(always)]
fn optimize_new<C: WordleConfig>(config: C, words: &Vec<[u8; 5]>) -> Vec<([u8; 5], f64)> {
    let cache = DashMap::new();
    let done = Mutex::new(0usize);
    let subwords: Vec<_> = words.clone().into_iter().filter(|&w| config.matches_word(w)).collect();
    let mut guesses = subwords.par_iter().map(|&guess| {
        let mut done = done.lock().unwrap();
        *done += 1;
        if *done % 100 == 0 {
            println!("{}/{}", *done, subwords.len());
        }
        drop(done);

        let (count, sum) = subwords.iter().map(|&correct| {
            let word_config = WordleConfig::from_guess_and_correct(guess, correct);
            let config_new = config.merge(word_config);

            if let Some(v) = cache.get(&config_new) {
                *v
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
    guesses
}

