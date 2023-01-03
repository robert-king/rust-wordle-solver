mod simple;
mod words;

use std::collections::{HashMap};

fn main() {
    let words = words::get_words(100, false);
    let mut results = vec![];
    let mut cache = HashMap::new();
    for &guess in &words {
        let avg = simple::evaluate_guess(guess, &words, &mut cache);
        results.push((avg, guess));
    }
    results.sort_by(|a, b| {
        a.0.partial_cmp(&b.0).unwrap()
    });
    for i in 0..10 {
        println!("{:?}", results[i]);
    }
}