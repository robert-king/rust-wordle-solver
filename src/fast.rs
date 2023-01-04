use std::collections::HashMap;
use rand::{thread_rng, Rng};

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
            *r = rng.gen_range(0..10u64.pow(13));
        }
        FastSolver{
            cache: HashMap::new(),
            all_words,
            buffer_pool: vec![],
            rand,
            is_valid_cache
        }
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
                new_idxs.extend(idxs
                    .iter()
                    .cloned()
                    .filter(|&word_idx| {
                        self.is_valid_cache[guess_idx * n * n + ans_idx * n + word_idx]
                    }));
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

    fn is_valid(w: &str, guess: &str, ans: &str) -> bool {
        let w = w.as_bytes();
        let guess = guess.as_bytes();
        let ans = ans.as_bytes();
        for i in 0..5 {
            if let Some(idx) = ans.iter().position(|&b| b == guess[i]) {
                if idx == i {
                    // green
                    if w[i] != ans[i] {
                        return false;
                    }
                } else {
                    // todo: make this work for duplicate characters
                    // orange
                    if !w.contains(&guess[i]) {
                        return false;
                    }
                    if w[i] == guess[i] {
                        return false;
                    }
                }
            } else {
                // guess[i] is not valid
                if w.contains(&guess[i]) {
                    return false;
                }
            }
        }
        true
    }

    fn get_valid_cache(all_words: &Vec<&'static str>) -> Vec<bool> {
        let n = all_words.len();
        let mut v = vec![false; n*n*n];
        let n = all_words.len();
        for (guess_idx, guess) in all_words.iter().enumerate() {
            println!("building for guess: {guess}");
            for (ans_idx, ans) in all_words.iter().enumerate() {
                for (word_idx, word) in all_words.iter().enumerate() {
                    v[guess_idx * n * n + ans_idx * n + word_idx] = FastSolver::is_valid(word, guess, ans);
                }
            }
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;
    use crate::{fast, words};
    use crate::fast::FastSolver;

    #[test]
    fn test_fast() {
        let guess = "trace";
        let words = words::get_words(10, false);
        let mut solver = FastSolver::new(words);
        let avg = solver.evaluate_guess(guess);
        assert_eq!(avg, 2.0);

        let words = words::get_words(100, false);
        let mut solver = FastSolver::new(words);
        let avg = solver.evaluate_guess(guess);
        assert_eq!(avg, 2.660000000000001);

        // benchmark simple
        use std::time::Instant;
        let now = Instant::now();

        // Code block to measure.
        {
            let words = words::get_words(80, false);
            let mut solver = FastSolver::new(words);
            for &guess in &words {
                let avg = solver.evaluate_guess(guess);
            }
        }

        let elapsed = now.elapsed();
        assert!(elapsed < Duration::new(3, 0));
        println!("Elapsed for 80 words: {:.2?}", elapsed);
    }
}
