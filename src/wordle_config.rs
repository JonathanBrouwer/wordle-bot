use std::fmt::Debug;
use std::hash::Hash;

pub trait WordleConfig : Debug + Copy + Clone + Eq + PartialEq + Hash + Send + Sync {
    fn default() -> Self;
    fn merge(self: Self, other: Self) -> Self;
    fn from_guess_and_correct(guess: [u8; 5], correct: [u8; 5]) -> Self;
    fn matches_word(&self, word: [u8; 5]) -> bool;
    fn is_finished(&self) -> bool;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SimpleWordleConfig {
    positions: [Option<u8>; 5],
    freqs_min: [u8; 26],
    freqs_exact: [bool; 26]
}

impl WordleConfig for SimpleWordleConfig {
    #[inline(always)]
    fn default() -> Self {
        SimpleWordleConfig {
            positions: [None; 5],
            freqs_min: [0; 26],
            freqs_exact: [false; 26]
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
    fn from_guess_and_correct(guess: [u8; 5], correct: [u8; 5]) -> Self {
        Self {
            positions: guess.zip(correct).map(|(gc, cc)| if gc == cc { Some(gc) } else { None }),
            freqs_min: word_freqs(&guess).zip(word_freqs(&correct)).map(|(g, c)| g.min(c)),
            freqs_exact: word_freqs(&guess).zip(word_freqs(&correct)).map(|(g, c)| g > c),
        }
    }

    #[inline(always)]
    fn matches_word(&self, word: [u8; 5]) -> bool {
        //Check positions
        if !self.positions.zip(word).into_iter().all(|(mc, c)| mc.is_none() || mc.unwrap() == c) { return false }

        //Check freqs
        return word_freqs(&word).zip(self.freqs_min.zip(self.freqs_exact)).into_iter().all(|(freq, (freq_min, freq_exact))| freq_min <= freq && (!freq_exact || freq_min == freq))
    }

    #[inline(always)]
    fn is_finished(&self) -> bool {
        self.positions.iter().all(|c| c.is_some())
    }
}



#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ComplexWordleConfig {
    pub(crate) positions: [[bool; 26]; 5],
    pub(crate) freqs_min: [u8; 26],
    pub(crate) freqs_exact: [bool; 26]
}

impl WordleConfig for ComplexWordleConfig {
    #[inline(always)]
    fn default() -> Self {
        Self {
            positions: [[true; 26]; 5],
            freqs_min: [0; 26],
            freqs_exact: [false; 26]
        }
    }

    #[inline(always)]
    fn merge(self: Self, other: Self) -> Self {
        Self {
            positions: self.positions.zip(other.positions).map(|(pa, pb)| pa.zip(pb).map(|(a, b)| a && b)),
            freqs_min: self.freqs_min.zip(other.freqs_min).map(|(a, b)| a.max(b)),
            freqs_exact: self.freqs_exact.zip(other.freqs_exact).map(|(a, b)| a || b),
        }
    }

    #[inline(always)]
    fn from_guess_and_correct(guess: [u8; 5], correct: [u8; 5]) -> Self {
        let mut positions = [[true; 26]; 5];
        let freqs_guess = word_freqs(&guess);
        let freqs_correct = word_freqs(&correct);
        for i in 0..5 {
            //Char is correct
            if guess[i] == correct[i] {
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
        Self {
            positions,
            freqs_min: freqs_guess.zip(freqs_correct).map(|(g, c)| g.min(c)),
            freqs_exact: freqs_guess.zip(freqs_correct).map(|(g, c)| g > c),
        }
    }

    #[inline(always)]
    fn matches_word(&self, word: [u8; 5]) -> bool {
        if !self.positions.zip(word).into_iter().all(|(poss, ch)| poss[num(ch) as usize]) { return false }
        return word_freqs(&word).zip(self.freqs_min).into_iter().all(|(freq, freq_min)| freq_min <= freq)
    }

    #[inline(always)]
    fn is_finished(&self) -> bool {
        self.positions.into_iter().all(|pos| pos.into_iter().filter(|&b| b).count() == 1)
    }
}

#[inline(always)]
fn word_freqs(word: &[u8; 5]) -> [u8; 26] {
    let mut in_word = [0u8; 26];
    word.iter().for_each(|letter| in_word[(*letter as usize) - ('a' as usize)] += 1);
    in_word
}

#[inline(always)]
pub fn num(c: u8) -> u8 {
    c - 'a' as u8
}