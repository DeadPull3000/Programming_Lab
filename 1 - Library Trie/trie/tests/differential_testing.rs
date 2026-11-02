//! Differential and randomized testing for Trie using a reference HashSet.

use std::collections::HashSet;
use trie::{normalize_word, Trie, TrieError};

/// A fast, zero-dependency deterministic pseudo-random number generator (Xorshift64*).
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x853c_49e6_748f_ea9b
            } else {
                seed
            },
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn next_range(&mut self, min: usize, max: usize) -> usize {
        let span = max - min + 1;
        min + (self.next_u64() as usize % span)
    }

    fn random_word(&mut self, min_len: usize, max_len: usize, alphabet: &[u8]) -> String {
        let len = self.next_range(min_len, max_len);
        let mut s = String::with_capacity(len);
        for _ in 0..len {
            let idx = self.next_range(0, alphabet.len() - 1);
            s.push(alphabet[idx] as char);
        }
        s
    }
}

#[test]
fn test_differential_randomized_operations() {
    // Fixed seed ensures 100% reproducible execution
    let mut rng = SimpleRng::new(0xDEAD_BEEF_CAFE_BABE);
    let mut trie = Trie::new();
    let mut reference = HashSet::<String>::new();

    // Use a small alphabet to maximize prefix collisions and shared tree paths
    let alphabet = b"ABCDE";
    let mut word_pool: Vec<String> = Vec::new();
    for _ in 0..60 {
        word_pool.push(rng.random_word(1, 6, alphabet));
    }

    for step in 0..2000 {
        let op_type = rng.next_range(0, 3);
        let word = if !word_pool.is_empty() && rng.next_range(0, 10) < 8 {
            let idx = rng.next_range(0, word_pool.len() - 1);
            word_pool[idx].clone()
        } else {
            rng.random_word(1, 6, alphabet)
        };

        match op_type {
            0 => {
                // Insert
                let trie_res = trie.insert(&word).expect("valid word must insert");
                let ref_res = reference.insert(word.clone());
                assert_eq!(
                    trie_res, ref_res,
                    "Step {step}: insert mismatch for '{word}'"
                );
            }
            1 => {
                // Remove
                let trie_res = trie.remove(&word).expect("valid word must remove");
                let ref_res = reference.remove(&word);
                assert_eq!(
                    trie_res, ref_res,
                    "Step {step}: remove mismatch for '{word}'"
                );
            }
            2 => {
                // Contains
                let trie_res = trie.contains(&word);
                let ref_res = reference.contains(&word);
                assert_eq!(
                    trie_res, ref_res,
                    "Step {step}: contains mismatch for '{word}'"
                );
            }
            3 => {
                // Prefix check
                let prefix_len = rng.next_range(1, word.len());
                let prefix = &word[..prefix_len];
                let trie_has_prefix = trie.has_prefix(prefix);
                let ref_has_prefix = reference.iter().any(|w| w.starts_with(prefix));
                assert_eq!(
                    trie_has_prefix, ref_has_prefix,
                    "Step {step}: prefix mismatch for prefix '{prefix}'"
                );
            }
            _ => unreachable!(),
        }

        // Verify invariants after every single operation
        assert_eq!(
            trie.len(),
            reference.len(),
            "Step {step}: word count mismatch"
        );
        assert_eq!(
            trie.is_empty(),
            reference.is_empty(),
            "Step {step}: is_empty mismatch"
        );
    }

    // Final sweep: verify every remaining reference word is found
    for w in &reference {
        assert!(trie.contains(w), "Final sweep: missing word '{w}'");
    }
}

#[test]
fn test_alphabet_boundary_extremities() {
    let mut trie = Trie::new();
    assert_eq!(trie.insert("A"), Ok(true));
    assert_eq!(trie.insert("Z"), Ok(true));
    assert_eq!(trie.insert("AAAA"), Ok(true));
    assert_eq!(trie.insert("ZZZZ"), Ok(true));
    assert_eq!(trie.insert("AM"), Ok(true));
    assert_eq!(trie.insert("ZM"), Ok(true));

    assert!(trie.contains("a"));
    assert!(trie.contains("z"));
    assert!(trie.contains("aaaa"));
    assert!(trie.contains("zzzz"));
    assert!(trie.contains("am"));
    assert!(trie.contains("zm"));

    assert_eq!(trie.remove("a"), Ok(true));
    assert!(!trie.contains("a"));
    assert!(trie.contains("aaaa"));
    assert!(trie.has_prefix("a"));

    assert_eq!(trie.remove("zzzz"), Ok(true));
    assert!(!trie.contains("zzzz"));
    assert!(trie.contains("z"));
    assert!(trie.contains("zm"));
}

#[test]
fn test_boundary_error_handling() {
    let mut trie = Trie::new();

    // Empty word errors
    assert_eq!(trie.insert(""), Err(TrieError::EmptyWord));
    assert_eq!(trie.insert("   \t\n  "), Err(TrieError::EmptyWord));
    assert_eq!(trie.remove(""), Err(TrieError::EmptyWord));

    // Invalid character errors
    assert_eq!(
        trie.insert("ABC1DEF"),
        Err(TrieError::InvalidCharacter('1'))
    );
    assert_eq!(
        trie.insert("HELLO WORLD"),
        Err(TrieError::InvalidCharacter(' '))
    );
    assert_eq!(trie.insert("CAN'T"), Err(TrieError::InvalidCharacter('\'')));
    assert_eq!(trie.remove("BAD!"), Err(TrieError::InvalidCharacter('!')));

    // Normalization edge cases
    assert_eq!(normalize_word("  valid  ").unwrap(), "VALID");
    assert_eq!(normalize_word(""), Err(TrieError::EmptyWord));
    assert_eq!(normalize_word("   "), Err(TrieError::EmptyWord));

    assert_eq!(trie.len(), 0);
    assert!(trie.is_empty());
}
