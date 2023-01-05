// mod simple;
mod words;
mod fast;

// fn run_simple(n: usize) {
//     let words = words::get_words(n, false);
//     let mut results = vec![];
//     let mut cache = HashMap::new();
//     for &guess in &words {
//         let avg = simple::evaluate_guess(guess, &words, &mut cache);
//         println!("{guess} {avg}");
//         results.push((avg, guess));
//     }
//     results.sort_by(|a, b| {
//         a.0.partial_cmp(&b.0).unwrap()
//     });
//     for i in 0..10 {
//         println!("simple: {:?}", results[i]);
//     }
// }

fn run_fast(n: usize) {
    let words = words::get_words(n, false);
    fast::FastSolver::anneal(words);
}

fn main() {
    // run_simple(10);
    run_fast(2000);
}