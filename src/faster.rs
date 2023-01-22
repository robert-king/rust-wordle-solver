use std::collections::{HashMap};

pub fn run_faster() {
    let n = 200;
    let words = crate::words::get_words(n, true)
        .iter()
        .map(|&w| Word::new(w))
        .collect::<Vec<Word>>();


    let (word_pattern_valid, ans_guess_pattern) = build_patterns(&words);

    /*
    for ans, guess pair to filter words:
    case1: nothing in common between guess and ans, then check nothing in common between word an guess.
    case2: something in common between guess and ans, get pattern, get valid words of that pattern,
     */
}

/*
word_pattern_valid[word_idx][pattern_idx] == true if word is valid given the pattern.
ans_guess_pattern[ans_idx][guess_idx] == pattern_idx of pattern generated when guessing word[guess_idx] against answer word[ans_idx]
 */
fn build_patterns(words: &Vec<Word>) -> (Vec<Vec<bool>>, Vec<Vec<usize>>) {
    let mut pattern_count = 1usize;
    let mut ans_guess_pattern = vec![vec![0; words.len()]; words.len()];
    let mut pattern_map = HashMap::new();
    let mut m: HashMap<u32, HashMap<u32, HashMap<[u8;5], HashMap<[u8;15], usize>>>> = HashMap::new();
    let letter_to_word_idx_lookup = get_letter_to_word_idx_lookup(&words);
    for (ans_idx, ans) in words.iter().enumerate() { // todo: could paralise this
        for b in 0..26 {
            if (ans.unique_bytes >> b) & 1 == 0 {
                continue;
            }
            for &guess_idx in &letter_to_word_idx_lookup[b as usize] {
                if ans_guess_pattern[ans_idx][guess_idx] != 0 {
                    continue;
                }
                let pat = Pattern::new(ans, &words[guess_idx]);
                ans_guess_pattern[ans_idx][guess_idx] = if let Some(&pattern_idx) = pattern_map.get(&pat) {
                     pattern_idx
                } else {
                    pattern_map.insert(pat, pattern_count);
                    let pat = Pattern::new(ans, &words[guess_idx]);
                    let e = m.entry(pat.shared).or_default();
                    let e2 = e.entry(pat.missing).or_default();
                    let e3 = e2.entry(pat.green).or_default();
                    assert!(e3.insert(pat.orange, pattern_count).is_none());
                    pattern_count += 1;
                    pattern_count - 1
                };
            }
        }
    }
    println!("Patterns len {}", pattern_map.len());
    let mut word_pattern_valid = vec![vec![false; pattern_map.len()+1]; words.len()];
    println!("calcing patters valid");
    let mut tot = 0;
    for (shared, e) in m {
        let words_e = (0..words.len()).filter(|&z| words[z].unique_bytes & shared == shared).collect::<Vec<usize>>();
        for (missing, e2) in e {
            let words_e2 = words_e.iter().filter(|&&z| words[z].unique_bytes & missing == 0).cloned().collect::<Vec<usize>>();
            for (green, e3) in e2 {
                let words_e3 = words_e2.iter().filter(|&&z| {
                    let w = &words[z];
                    let mut shared = shared;
                    let mut i = 0;
                    while shared != 0 {
                        let letter_idx = shared.trailing_zeros() as usize;
                        if green[i] & w.positions[letter_idx] != green[i] {
                            return false;
                        }
                        shared -= 1 << letter_idx;
                        i += 1;
                    }
                    true
                }).cloned().collect::<Vec<usize>>();
                for (orange, pattern_idx) in e3 {
                    let words_e4 = words_e3.iter().filter(|&&z| {
                        let w = &words[z];
                        let mut shared = shared;
                        let mut i = 0;
                        while shared != 0 {
                            let letter_idx = shared.trailing_zeros() as usize;
                            let r = w.positions[letter_idx] - green[i];
                            if r & orange[i] != 0 {
                                // if we match an orange position, we too are orange
                                return false;
                            }

                            let count = w.positions[letter_idx].count_ones() as u8;
                            if count < orange[i+5] || count > orange[i+10] {
                                // too few or too many based on min and max count.
                                return false;
                            }

                            shared -= 1 << letter_idx;
                            i += 1;
                        }
                        true
                    }).cloned().collect::<Vec<usize>>();
                    for word_idx in words_e4 {
                        word_pattern_valid[word_idx][pattern_idx] = true;
                    }
                }
            }
        }
    }

    return (word_pattern_valid, ans_guess_pattern)

}

fn get_letter_to_word_idx_lookup(words: &Vec<Word>) -> Vec<Vec<usize>> {
    let mut letter_to_word_idx_lookup = vec![vec![]; 26];
    for (word_idx, word) in words.iter().enumerate() {
        for &b in &word.bytes {
            if letter_to_word_idx_lookup[b as usize].last() != Some(&word_idx) {
                letter_to_word_idx_lookup[b as usize].push(word_idx);
            }
        }
    }
    letter_to_word_idx_lookup
}


struct Word {
    bytes: [u8; 5],
    positions: [u8; 26], // bitmask of locations of that letter, e.g. for aabce, positions = 00011, 00100, 01000, 00000, 10000,....
    unique_bytes: u32,
    w: String,
}

impl Word {
    fn new(word: &'static str) -> Self {
        let mut unique_bytes = 0;
        let mut bytes = [0; 5];
        for i in 0..5 {
            bytes[i] = word.as_bytes()[i] - b'a';
            unique_bytes |= 1 << bytes[i];
        }
        let mut positions = [0u8; 26];
        for (i, &b) in bytes.iter().enumerate() {
            positions[b as usize] |= 1 << (i as u8);
        }
        Word {
            bytes,
            positions,
            unique_bytes,
            w: word.to_string()
        }
    }
}

#[derive(Default, Hash, PartialEq, Eq, Debug)]
struct Pattern {
    missing: u32,
    shared: u32,  // for each bit of shared, we get a letter idx (shared[i])
    green: [u8; 5], // green[i] is the bits that are green for letter corresponding to shared[i]
    orange: [u8; 15], // first five is bits that could be orange, second five is minimum counts, third five is max counts
}

impl Pattern {
    fn new(ans: &Word, guess: &Word) -> Self {
        let mut pat = Pattern::default();
        let mut shared = guess.unique_bytes & ans.unique_bytes;
        pat.shared = shared;
        pat.missing = guess.unique_bytes ^ shared;
        let mut i = 0;
        while shared != 0 {
            let letter_idx = shared.trailing_zeros() as u8;
            let x = ans.positions[letter_idx as usize];
            let y = guess.positions[letter_idx as usize];
            let green = x & y;
            pat.green[i] = green;
            let orange = green ^ y;
            pat.orange[i] = orange;
            let min_count = (y.count_ones() as u8).min(x.count_ones() as u8);
            pat.orange[i+5] = min_count;

            let mut max_count = 5;
            let max_count = if y.count_ones() > x.count_ones() {
                x.count_ones() as u8
            } else {
                5
            };
            pat.orange[i+10] = max_count;

            shared -= 1 << letter_idx;
            i += 1;
        }
        pat
    }
}


#[cfg(test)]
mod tests {
    use crate::faster::Word;

    #[test]
    fn test_word() {
        let w1 = Word::new("aaabc");
        let w2 = Word::new("aabyz");
        let mut common = w1.unique_bytes & w2.unique_bytes;
        assert_eq!(common, 0b11);
        let mut v_green = vec![];
        while common != 0 {
            let j = common.trailing_zeros() as usize;
            let green = w1.positions[j] & w2.positions[j];
            v_green.push(green);
            common -= 1 << j;
        }
        assert_eq!(v_green, vec![0b11, 0b0]);
    }
}