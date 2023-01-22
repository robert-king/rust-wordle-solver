use std::collections::{HashMap, HashSet};
use rand::{thread_rng, Rng, seq::SliceRandom};
use rayon::prelude::*;

pub fn run_fast(n: usize) {
    let words = crate::words::get_words(n, false);
    FastSolver::anneal(words);
}

pub struct FastSolver {
    cache: HashMap<u64, f64>,
    all_words: Vec<&'static str>,
    buffer_pool: Vec<Vec<usize>>,
    rand: Vec<u64>,
    is_valid_cache: Vec<bool>
}

impl FastSolver {
    pub fn new(all_words: Vec<&'static str>) -> Self {
        let is_valid_cache = FastSolver::get_valid_cache(&all_words);
        let n = all_words.len();
        let mut rand = vec![0; n];
        let mut rng = thread_rng();
        for r in &mut rand {
            *r = rng.gen_range(0..10u64.pow(15));
        }
        FastSolver{
            cache: HashMap::new(),
            all_words,
            buffer_pool: vec![],
            rand,
            is_valid_cache
        }
    }

    pub fn anneal(mut all_words: Vec<&'static str>) {
        all_words.shuffle(&mut thread_rng());
        let mut top_words = all_words.clone();
        let n = all_words.len();
        let chunk_sizes = vec![n/10, n/5, n/2];
        for chunk_size in chunk_sizes {
            let initial_guesses = HashSet::<&'static str>::from_iter(top_words.iter().cloned());
            top_words = all_words.par_iter().cloned().chunks(chunk_size).map(|words_chunk| {
                let mut solver = FastSolver::new(words_chunk);
                solver.evaluate_guesses(&initial_guesses).iter().take(30).map(|(_avg, guess)| guess).cloned().collect::<Vec<&'static str>>()
            }).flatten_iter().collect::<Vec<&'static str>>();
            println!("done {}", top_words.len());
        }

        let initial_guesses = HashSet::<&'static str>::from_iter(top_words.iter().cloned());
        println!("building final solver");
        let mut final_solver = FastSolver::new(all_words);
        println!("running final solver");
        let results = final_solver.evaluate_guesses(&initial_guesses);
        for result in results.iter().take(5) {
            println!("{result:?}");
        }
    }

    fn evaluate_guesses(&mut self, initial_guesses: &HashSet<&'static str>) -> Vec<(f64, &'static str)> {
        let mut results = vec![];
        for (guess_idx, &guess) in self.all_words.clone().iter().enumerate() {
            if initial_guesses.contains(guess) {
                let avg = self.evaluate_one_guess(guess_idx);
                // println!("{guess} {avg}");
                results.push((avg, guess));
            }
        }
        results.sort_by(|a, b| {
            a.0.partial_cmp(&b.0).unwrap()
        });
        results
    }

    pub fn evaluate_one_guess(&mut self, guess_idx: usize) -> f64 {
        let mut idxs = self.buffer_pool.pop().unwrap_or_default();
        idxs.clear();
        idxs.extend(0..self.all_words.len());
        self.evaluate_guess(guess_idx, &idxs)
    }

    fn evaluate_guess(&mut self, guess_idx: usize, idxs: &Vec<usize>) -> f64 {
        let n = self.all_words.len();
        let mut total = 0f64;
        let mut new_idxs = self.buffer_pool.pop().unwrap_or_default();
        for &ans_idx in idxs {
            total += 1.0;
            if ans_idx != guess_idx {
                new_idxs.clear();
                new_idxs.extend(
                    idxs.iter()
                        .filter(|&word_idx| {
                            self.is_valid_cache[guess_idx * n * n + ans_idx * n + word_idx]
                        })
                );
                total += match new_idxs.len() {
                    1 => 1.0,
                    2 => 2.5,
                    _ => self.evaluate_next_guess(&new_idxs)
                }
            }
        }
        self.buffer_pool.push(new_idxs);
        total / (idxs.len() as f64)
    }

    fn evaluate_next_guess(&mut self, idxs: &Vec<usize>) -> f64 {
        let hash = idxs.iter().map(|&idx| self.rand[idx]).sum();
        if let Some(&ans) = self.cache.get(&hash) {
            return ans;
        }
        let mut best_avg = f64::MAX;
        // todo: perhaps we could make guesses outside of words
        for &guess_idx in idxs {
            let avg = self.evaluate_guess(guess_idx, &idxs);
            best_avg = best_avg.min(avg);
        }
        self.cache.insert(hash, best_avg);
        best_avg
    }

    fn get_valid_cache(all_words: &Vec<&'static str>) -> Vec<bool> {
        let words = all_words.iter().map(|&w| Word::new(w)).collect::<Vec<Word>>();
        let n = all_words.len();
        let mut v = vec![false; n*n*n];
        // todo: we could make this faster by computing matchable patterns first for each word, and then only looking at relevant words.
        v.par_iter_mut().chunks(n).enumerate().for_each(|(chunk_idx, chunk)| {
            let guess_idx = chunk_idx / n;
            let guess = &words[guess_idx];
            let ans_idx = chunk_idx % n;
            let ans = &words[ans_idx];
            for (is_valid, word) in chunk.into_iter().zip(&words) {
                if word.is_valid(guess, ans) {
                    *is_valid = true;
                }
            }
        });
        v
    }
}

struct Word {
    bytes: [u8; 5],
    position: [u8; 26],
}

impl Word {
    fn new(word: &'static str) -> Self {
        let mut bytes = [0; 5];
        for i in 0..5 {
            bytes[i] = word.as_bytes()[i] - b'a';
        }
        let mut position = [5; 26];
        for (i, &b) in bytes.iter().enumerate() {
            position[b as usize] = i as u8;
        }
        Word {
            bytes,
            position
        }
    }

    fn contains(&self, b: u8) -> bool {
        self.position[b as usize] != 5
    }

    fn is_valid(&self, guess: &Word, ans: &Word) -> bool {
        for (i, &guess_b) in guess.bytes.iter().enumerate() {
            let idx = ans.position[guess_b as usize];
            if idx != 5 {
                if idx == i as u8 {
                    // green
                    if self.bytes[i] != guess_b {
                        return false;
                    }
                } else {
                    // todo: make this work for duplicate characters
                    // orange
                    if !self.contains(guess_b) {
                        return false;
                    }
                    if self.bytes[i] == guess_b {
                        return false;
                    }
                }
            } else {
                // guess[i] is not valid
                if self.contains(guess_b) {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;
    use crate::{fast, words};
    use crate::fast::FastSolver;

    #[test]
    fn test_is_valid() {
        let guess = fast::Word::new("stuck");
        let word = fast::Word::new("pluck");

        assert!(word.is_valid(&fast::Word::new("xzuck"), &guess));
        assert!(!word.is_valid(&fast::Word::new("truck"), &guess));
    }

    // #[test]
    // fn test_fast() {
    //     let guess = "trace";
    //     let words = words::get_words(10, false);
    //     let mut solver = FastSolver::new(words);
    //     let avg = solver.evaluate_guess(guess);
    //     assert_eq!(avg, 2.0);
    //
    //     let words = words::get_words(100, false);
    //     let mut solver = FastSolver::new(words);
    //     let avg = solver.evaluate_guess(guess);
    //     assert_eq!(avg, 2.660000000000001);
    //
    //     // benchmark simple
    //     use std::time::Instant;
    //     let now = Instant::now();
    //
    //     // Code block to measure.
    //     {
    //         let words = words::get_words(80, false);
    //         let mut solver = FastSolver::new(words);
    //         for &guess in &words {
    //             let avg = solver.evaluate_guess(guess);
    //         }
    //     }
    //
    //     let elapsed = now.elapsed();
    //     assert!(elapsed < Duration::new(3, 0));
    //     println!("Elapsed for 80 words: {:.2?}", elapsed);
    // }
}
