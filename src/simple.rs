use std::collections::HashMap;
use std::time::Duration;
use crate::words;


pub fn evaluate_guess(guess: &str, words: &Vec<&'static str>, cache: &mut HashMap<Vec<&'static str>, f64>) -> f64 {
    let mut total = 0f64;
    for &ans in words {
        total += 1.0;
        if ans != guess {
            let new_words = words
                .iter()
                .cloned()
                .filter(|&w| is_valid(w, guess, ans))
                .collect::<Vec<&str>>();
            total += match new_words.len() {
                1 => 1.0,
                2 => 2.5,
                _ => evaluate_next_guess(new_words, cache)
            }
        }
    }
    total / (words.len() as f64)
}

fn evaluate_next_guess(words: Vec<&'static str>, cache: &mut HashMap<Vec<&'static str>, f64>) -> f64 {
    if let Some(&ans) = cache.get(&words) {
        return ans;
    }
    let mut best_avg = f64::MAX;
    // todo: perhaps we could make guesses outside of words
    for &guess in &words {
        let avg = evaluate_guess(guess, &words, cache);
        best_avg = best_avg.min(avg);
    }
    cache.insert(words, best_avg);
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

#[test]
fn test_simple() {
    let guess = "trace";
    let words = words::get_words(10, false);
    let mut cache = HashMap::new();
    let avg = evaluate_guess(guess, &words, &mut cache);
    assert_eq!(avg, 2.0);

    let words = words::get_words(100, false);
    let mut cache = HashMap::new();
    let avg = evaluate_guess(guess, &words, &mut cache);
    assert_eq!(avg, 2.660000000000001);

    // benchmark simple
    use std::time::Instant;
    let now = Instant::now();

    // Code block to measure.
    {
        let words = words::get_words(80, false);
        let mut cache = HashMap::new();
        for &guess in &words {
            let _avg = evaluate_guess(guess, &words, &mut cache);
        }
    }

    let elapsed = now.elapsed();
    assert!(elapsed < Duration::new(3, 0));
    println!("Elapsed for 80 words: {:.2?}", elapsed);
}