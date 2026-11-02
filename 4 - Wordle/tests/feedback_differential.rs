//! # Wordle Feedback Differential & Exhaustive Metamorphic Testing
//!
//! Provides exhaustive mathematical and randomized differential testing of
//! `calculate_feedback` against two independently written reference algorithms.
//!
//! Specifically targets **repeated letters**, where naive Wordle algorithms fail:
//! 1. **Exhaustive 3-Letter Alphabet Domain:** Evaluates all $3^5 \times 3^5 = 59,049$
//!    exhaustive combinations of answer and guess pairs over `['A', 'B', 'C']`.
//! 2. **High-Collision 4-Letter Alphabet Domain:** 10,000 seeded pairs over `['A', 'B', 'C', 'D']`.
//! 3. **Full 26-Letter Alphabet Domain:** 10,000 seeded pairs over `['A'..='Z']`.
//! 4. **Real Dictionary Lexicon Pairs:** 10,000 pairs sampled from `dictionary.txt`.
//! 5. **Pathological Hand-Crafted Repeated-Letter Cases:** Verified against known Wordle rules.

use std::collections::HashMap;
use wordle::feedback::{calculate_feedback, Feedback, LetterFeedback};
use wordle::word::Word;

// ============================================================================
// INDEPENDENT REFERENCE ALGORITHMS
// ============================================================================

/// Reference Algorithm 1: Direct Slot-Masking Array
///
/// Models letter consumption by placing the answer's bytes in an array of
/// `[Option<u8>; 5]`.
/// - Pass 1: Identifies exact matches, sets `available[i] = None`, marks `Correct`.
/// - Pass 2: Linearly scans `available` for each unmatched guess byte; if a match
///   is found, sets `available[slot] = None` and marks `Present`.
fn reference_algorithm_1_slot_masking(answer: &Word, guess: &Word) -> Feedback {
    let a = answer.as_bytes();
    let g = guess.as_bytes();
    let mut available = [Some(a[0]), Some(a[1]), Some(a[2]), Some(a[3]), Some(a[4])];
    let mut states = [LetterFeedback::Absent; 5];

    // Pass 1: Exact Green matches
    for i in 0..5 {
        if g[i] == a[i] {
            states[i] = LetterFeedback::Correct;
            available[i] = None; // consumed
        }
    }

    // Pass 2: Displaced Yellow matches
    for i in 0..5 {
        if states[i] == LetterFeedback::Absent {
            for slot in &mut available {
                if *slot == Some(g[i]) {
                    states[i] = LetterFeedback::Present;
                    *slot = None; // consumed
                    break;
                }
            }
        }
    }

    Feedback::new(states)
}

/// Reference Algorithm 2: Explicit Multiset Frequency Tracking
///
/// An alternative approach using hash map frequencies:
/// - Counts frequency of characters in `answer`.
/// - Pass 1: Discovers Green matches and decrements the frequency map.
/// - Pass 2: For non-Green positions, checks if the letter's count in the map > 0.
fn reference_algorithm_2_multiset(answer: &Word, guess: &Word) -> Feedback {
    let a = answer.as_bytes();
    let g = guess.as_bytes();
    let mut states = [LetterFeedback::Absent; 5];
    let mut answer_counts: HashMap<u8, usize> = HashMap::new();

    for &b in a {
        *answer_counts.entry(b).or_insert(0) += 1;
    }

    // Pass 1: Green matches take highest priority
    for i in 0..5 {
        if g[i] == a[i] {
            states[i] = LetterFeedback::Correct;
            let count = answer_counts.get_mut(&g[i]).unwrap();
            *count -= 1;
        }
    }

    // Pass 2: Yellow matches allocate remaining frequencies
    for i in 0..5 {
        if states[i] != LetterFeedback::Correct {
            if let Some(count) = answer_counts.get_mut(&g[i]) {
                if *count > 0 {
                    states[i] = LetterFeedback::Present;
                    *count -= 1;
                }
            }
        }
    }

    Feedback::new(states)
}

// ============================================================================
// VERIFICATION HARNESS & INVARIANT ASSERTIONS
// ============================================================================

/// Asserts that all algorithms agree and all mathematical Wordle invariants hold.
fn verify_pair(answer: &Word, guess: &Word) {
    let prod = calculate_feedback(answer, guess);
    let ref1 = reference_algorithm_1_slot_masking(answer, guess);
    let ref2 = reference_algorithm_2_multiset(answer, guess);

    // 1. Triple-algorithm agreement
    assert_eq!(
        prod,
        ref1,
        "Mismatch between production and Reference 1 for answer='{}', guess='{}'",
        answer.as_str(),
        guess.as_str()
    );
    assert_eq!(
        prod,
        ref2,
        "Mismatch between production and Reference 2 for answer='{}', guess='{}'",
        answer.as_str(),
        guess.as_str()
    );

    let a_bytes = answer.as_bytes();
    let g_bytes = guess.as_bytes();
    let states = prod.as_slice();

    // 2. Invariant: Green if and only if characters are identical at position i
    for i in 0..5 {
        if states[i] == LetterFeedback::Correct {
            assert_eq!(
                g_bytes[i], a_bytes[i],
                "Tile marked Green but characters differed at pos {i}: g='{}', a='{}'",
                g_bytes[i] as char, a_bytes[i] as char
            );
        } else {
            assert_ne!(
                g_bytes[i], a_bytes[i],
                "Characters matched at pos {i} but tile was not Green: g='{}', a='{}'",
                g_bytes[i] as char, a_bytes[i] as char
            );
        }
    }

    // 3. Invariant: Yellow letters MUST appear in the answer
    for i in 0..5 {
        if states[i] == LetterFeedback::Present {
            assert!(
                a_bytes.contains(&g_bytes[i]),
                "Tile marked Yellow but letter '{}' does not exist in answer '{}'",
                g_bytes[i] as char,
                answer.as_str()
            );
        }
    }

    // 4. Invariant: Frequency Conservation (CRITICAL REPEATED-LETTER INVARIANT)
    // For every letter, count(G) + count(Y) <= count_in_answer
    let mut ans_counts = [0usize; 26];
    for &b in a_bytes {
        ans_counts[(b - b'A') as usize] += 1;
    }

    let mut allocated_counts = [0usize; 26];
    for i in 0..5 {
        if states[i] != LetterFeedback::Absent {
            let idx = (g_bytes[i] - b'A') as usize;
            allocated_counts[idx] += 1;
        }
    }

    for idx in 0..26 {
        assert!(
            allocated_counts[idx] <= ans_counts[idx],
            "Frequency conservation violated for letter '{}': allocated {} times, but answer '{}' only contains it {} times! (guess='{}', feedback='{}')",
            (b'A' + idx as u8) as char,
            allocated_counts[idx],
            answer.as_str(),
            ans_counts[idx],
            guess.as_str(),
            prod.to_string_pattern()
        );
    }
}

// ============================================================================
// TEST SUITES
// ============================================================================

#[test]
fn test_exhaustive_3_letter_alphabet_domain() {
    // Alphabet: 'A', 'B', 'C'
    // Total words = 3^5 = 243 words
    // Total pairs = 243 * 243 = 59,049 pairs
    // Exhaustively tests every single possible duplicate letter configuration!
    let alphabet = b"ABC";
    let mut words = Vec::with_capacity(243);

    for &c1 in alphabet {
        for &c2 in alphabet {
            for &c3 in alphabet {
                for &c4 in alphabet {
                    for &c5 in alphabet {
                        let bytes = [c1, c2, c3, c4, c5];
                        let s = std::str::from_utf8(&bytes).unwrap();
                        words.push(Word::parse(s).unwrap());
                    }
                }
            }
        }
    }

    assert_eq!(words.len(), 243);

    let start = std::time::Instant::now();
    let mut pairs_checked = 0;

    for answer in &words {
        for guess in &words {
            verify_pair(answer, guess);
            pairs_checked += 1;
        }
    }

    let elapsed = start.elapsed();
    println!(
        "[Exhaustive 3-Letter Test] Checked all {pairs_checked} answer/guess pairs in {elapsed:?} with 0 mismatches!"
    );
    assert_eq!(pairs_checked, 59_049);
}

#[test]
fn test_seeded_random_4_letter_alphabet_high_collision() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(0x1234_5678_9ABC_DEF0);
    let alphabet = b"ABCD";
    let iterations = 10_000;

    for _ in 0..iterations {
        let mut a_bytes = [0u8; 5];
        let mut g_bytes = [0u8; 5];
        for i in 0..5 {
            a_bytes[i] = alphabet[rng.gen_range(0..alphabet.len())];
            g_bytes[i] = alphabet[rng.gen_range(0..alphabet.len())];
        }

        let ans = Word::parse(std::str::from_utf8(&a_bytes).unwrap()).unwrap();
        let gss = Word::parse(std::str::from_utf8(&g_bytes).unwrap()).unwrap();

        verify_pair(&ans, &gss);
    }
}

#[test]
fn test_seeded_random_26_letter_alphabet() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let mut rng = StdRng::seed_from_u64(0xDEAD_BEEF_FEED_CAFE);
    let iterations = 10_000;

    for _ in 0..iterations {
        let mut a_bytes = [0u8; 5];
        let mut g_bytes = [0u8; 5];
        for i in 0..5 {
            a_bytes[i] = rng.gen_range(b'A'..=b'Z');
            g_bytes[i] = rng.gen_range(b'A'..=b'Z');
        }

        let ans = Word::parse(std::str::from_utf8(&a_bytes).unwrap()).unwrap();
        let gss = Word::parse(std::str::from_utf8(&g_bytes).unwrap()).unwrap();

        verify_pair(&ans, &gss);
    }
}

#[test]
fn test_seeded_dictionary_words_differential() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use wordle::dictionary::WordleDictionary;

    let dict = WordleDictionary::load_default().expect("load dictionary");
    let answers = dict.answers();
    assert!(!answers.is_empty());

    let mut rng = StdRng::seed_from_u64(0xCAFE_BABE_0123_4567);
    let iterations = 10_000;

    for _ in 0..iterations {
        let ans = &answers[rng.gen_range(0..answers.len())];
        let gss = &answers[rng.gen_range(0..answers.len())];
        verify_pair(ans, gss);
    }
}

#[test]
fn test_hand_crafted_pathological_repeated_letter_cases() {
    let cases = [
        // (Answer, Guess, Expected Pattern)
        ("APPLE", "APPLE", "GGGGG"),
        ("APPLE", "ALLEY", "GYXYX"),
        ("APPLE", "AAAAA", "GXXXX"),
        ("LEVEL", "ELVES", "YYGGX"),
        ("BANAL", "ALLAY", "YYXGX"),
        ("CRATE", "TRACE", "YGGYG"),
        ("SPEED", "ERASE", "YXXYY"),
        ("SPEED", "CREEP", "XXGGY"),
        ("GEESE", "EERIE", "YGXXG"),
        ("PUPPY", "PEPPY", "GXGGG"),
        ("NANNY", "ANNOY", "YYGXG"),
        ("KEEPS", "PEEKS", "YGGYG"),
        ("CHEER", "LEECH", "XYGYY"),
        ("BOBBY", "BABYS", "GXGYX"),
        ("TITAN", "ATLAS", "XYXGX"),
        ("SASSY", "ASSES", "YYGXY"),
        ("LLAMA", "MALLS", "YYYYX"),
        ("OTTER", "TORTE", "YYYYY"),
        ("MAMMA", "AMAMM", "YYYGY"),
        ("EERIE", "EERIE", "GGGGG"),
        ("EERIE", "ENTER", "GXXYY"),
        ("ROBOT", "BOOST", "YGYXG"),
        ("DADDY", "DREAD", "GXXYY"),
        ("SHEEP", "GEESE", "XYGYX"),
    ];

    for (ans_str, gss_str, expected) in cases {
        let ans = Word::parse(ans_str).unwrap();
        let gss = Word::parse(gss_str).unwrap();
        let feedback = calculate_feedback(&ans, &gss);

        assert_eq!(
            feedback.to_string_pattern(),
            expected,
            "Failed hand-crafted test case for answer='{ans_str}', guess='{gss_str}'"
        );
        verify_pair(&ans, &gss);
    }
}
