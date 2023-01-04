mod simple;
mod words;
mod fast;

use std::collections::{HashMap};

fn run_simple(n: usize) {
    let words = words::get_words(n, false);
    let mut results = vec![];
    let mut cache = HashMap::new();
    for &guess in &words {
        let avg = simple::evaluate_guess(guess, &words, &mut cache);
        println!("{guess} {avg}");
        results.push((avg, guess));
    }
    results.sort_by(|a, b| {
        a.0.partial_cmp(&b.0).unwrap()
    });
    for i in 0..10 {
        println!("simple: {:?}", results[i]);
    }
}

fn run_fast(n: usize) {
    let mut words = words::get_words(n, false);
    words.sort();
    let mut results = vec![];
    println!("building");
    let mut solver = fast::FastSolver::new(words.clone());
    // let mut idxs = (0..words.len()).collect::<Vec<usize>>();
    for (guess_idx, guess) in words.into_iter().enumerate() {
        let avg = solver.evaluate_one_guess(guess_idx);
        println!("{guess} {avg}");
        results.push((avg, guess));
    }
    results.sort_by(|a, b| {
        a.0.partial_cmp(&b.0).unwrap()
    });
    for i in 0..10 {
        println!("fast: {:?}", results[i]);
    }
}

fn main() {
    // run_simple(200);
    run_fast(200);
}