# Wordle Solver, Tutorial, Testing & Benchmarks

### High level goals:
- Build something! Brute force first
- Create a language agnostic tutorial so others can build a solver
- Showcase Rust tho, live code the solver on YouTube (https://www.youtube.com/@robertking)
- Create some tests so others can test
- Create some benchmarks so we can compare
- Let people find optimal initial guesses that use words they know.
- Optimise based on average expected number of guesses
- Higher quality code than previous terminal game wordle project (https://github.com/robert-king/rust-wordle)

### Tutorial Steps and Hints:

#### Prerequisite:
- This isn't easy, so have some tenacity, feel free to look at src/simple.rs for hints to unblock you, or watch the youtube video of RusyRob coding it up first and then try to repeat.
- You should be comfortable with recursion and some casework
- Understand taking the average value from a set of possible outcomes.
- Understand taking the minimum value of a set of possible options and use that as an outcome.

#### Steps: (see hints in the next section)
1) Thinks of a few approaches and write them down
2) Create a method called evaluate_guess that returns the average number of guesses required for a word. How does it calculate the average, what helper functions does it need to call.
3) Create `fn is_valid(w: &str, guess: &str, ans: &str) -> bool`, to simplify the logic which can be difficult, only consider words that don't have duplicate characters, later, filter your word-list to exclude words with duplicate characters.
4) Create `evaluate_next_guess(words: Vec<&'static str>) -> f64` 
5) Test `evaluate_guess()` on a few examples, the avg score of a guess should be between ~2.0 and 5.0, depending on how many initial words you have in your input
6) Think of some ways to improve performance
7) Improve accuracy by allowing duplicate characters and by allowing guesses from a larger list of words
8) Optimise performance further using [Flame Graph](https://github.com/flamegraph-rs/flamegraph)
9) when Optimising, create a struct called FastSolver that houses the cache and other structures used for optimisation, create a new file called fast.rs

#### Hints:
use src/words.rs and src/simple.rs as a guide, as well as the youtube video.

1) Go with a brute force approach (using recursion), we can optimise later with caching since there aren't many states due to limited colourings (3^5) of each pattern.
2) use the Signature `evaluate_guess(guess: &str, words: &Vec<&'static str>) -> f64` and return the average number of guesses across all possible answers from `words`. If an answer is equal to the guess, it should contribute `1.0/words.len()` towards the average, otherwise, use that answer to narrow down the list of candidate words, using a helper `is_valid(word, guess, ans)`, and call another helper e.g. `next_guess_avg = evaluate_next_guess(narrowed_words)`. Don't worry about the 5 guess limit, although you could return infinity if you reached a certain depth, we didn't bother.
3) for each of the 5 characters in guess, is it orange, green or black? In each of these three case, what does it tell us about if the current word is valid or not? N.B. in the orange case, it tells us at least two things :) (it depends on if you're allowing duplicate characters or not) 
4) In this method, for each possible guess, we should see which guess is the best, and return that as the best average. We can call our previous method `evaluate_guess()`.
5) You should start out with just a few words for testing and slowly increase the number of words to see where performance degrades. You can use `["cigar", "rebut", "blush", "focal", "trace"]` to start with.
6) The easiest way is to memoize `evaluate_next_guess` using a hashmap, e.g. `evaluate_next_guess(words: Vec<&'static str>, cache: &mut HashMap<Vec<&'static str>, f64>) -> f64` e.g. `if let Some(&ans) = cache.get(&words) {
   return ans;
   }` 
7) See the source code for the two list of words (one is a list of valid answers, the other is more obscure words that can be used as guesses, however I advise you use words that are part of your own vocabulary as they're most useful for your game play)
8) Lets discus ways to optimise together, however, here are a few ideas I have: reduce allocations, e.g. don't allocate more memory when caching or when filtering the words. Prune out bad words early, e.g. run your algorithm on a small subset of words and remove words that did badly, they will never be good guesses. (this technique is similar to simulated annealing?) 

### Testing:
- run `cargo test` use `fn test_simple()` to check your logic.
- more test cases are welcome! :)

### Benchmarking:
- You must use this command to allow printing: `cargo test -- --nocapture`
- could use some help here, but test_simple contains a benchmark for now using std::time::Instant;
- It will print `Elapsed for 80 words: 625.51ms`
- try "cargo install flamegraph" and "cargo flamegraph" (https://github.com/flamegraph-rs/flamegraph for help)

### Pull Requests:
- are most welcome :)

### Flamegraph iterations:
1) flamegraph1.svg: evaluate_guess is slow due to allocating vector. Lets try use partition  