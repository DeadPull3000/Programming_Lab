# Testing and Validation Report

This document reports the testing methodology, test suites, coverage matrix, differential testing results, and verification records for the Trie dictionary project.

---

## 1. Testing Methodology and Suite Taxonomy

Testing is organized into five distinct layers to guarantee correctness across unit, integration, randomized, and stress-testing dimensions:

1. **Unit Tests — Core Trie (`trie/src/lib.rs` - 29 tests)**:
   - Validates node initialization, constructors, normalization, single-word and batch operations, error reporting, case conversions, and deletion pruning.
2. **Differential & Randomized Tests (`trie/tests/differential_testing.rs` - 3 tests)**:
   - Compares Trie mutations and queries against a reference `std::collections::HashSet<String>` across 2,000 randomized operations.
   - Tests boundary extremities (`'A'`, `'Z'`, repeated letters).
3. **Batch & File I/O Tests (`trie-cli/src/batch.rs` - 7 tests)**:
   - Validates delimited parsing, empty readers, mixed line/comma delimiters, line-indexed malformed errors, candidate classification, and missing file errors.
4. **Interactive CLI Tests (`trie-cli/src/main.rs` - 7 tests)**:
   - Tests command dispatching (`find`, `insert`, `delete`, `found`, `missing`, `stats`, `help`, `quit`), whitespace inputs, and unknown commands.
   - Includes full-dictionary integration test (`test_full_dictionary_loading_and_lookup`) asserting total dictionary counts against an independently calculated HashSet.
5. **Documentation Tests (`trie/src/lib.rs` - 8 doctests)**:
   - Verifies that all examples in doc-comments compile and pass assertions.

**Total Verified Tests Across Workspace**: **54 tests** (all passing, 0 failures, 0 ignored).

---

## 2. Test Coverage Matrix

| Test ID | Test Name | Location | Scenario Tested | Input / Setup | Expected Result | Actual Result |
|---|---|---|---|---|---|---|
| **T01** | `test_new_trie_is_empty` | `trie/src/lib.rs` | New Trie initialization | `Trie::new()` | `len() == 0`, `is_empty() == true` | Passed (0.00s) |
| **T02** | `test_default_trie_is_empty` | `trie/src/lib.rs` | Default trait implementation | `Trie::default()` | `len() == 0`, `is_empty() == true` | Passed (0.00s) |
| **T03** | `test_node_initial_state` | `trie/src/lib.rs` | Internal Node initialization | `Node::new()` | `is_terminal == false`, all 26 children `None` | Passed (0.00s) |
| **T04** | `test_char_to_index_mapping` | `trie/src/lib.rs` | Char to 0-based index | `'A'`, `'B'`, `'Y'`, `'Z'`, `'a'`, `'1'`, `'!'` | `Some(0)`, `Some(1)`, `Some(24)`, `Some(25)`, `None`, `None`, `None` | Passed (0.00s) |
| **T05** | `test_index_to_char_mapping` | `trie/src/lib.rs` | Index to char | `0`, `1`, `24`, `25`, `26`, `100` | `Some('A')`, `Some('B')`, `Some('Y')`, `Some('Z')`, `None`, `None` | Passed (0.00s) |
| **T06** | `test_round_trip_index_char` | `trie/src/lib.rs` | Round-trip bijective mapping | All `0..=25` | `char_to_index(index_to_char(i)) == Some(i)` | Passed (0.00s) |
| **T07** | `test_normalize_valid_words` | `trie/src/lib.rs` | Case and whitespace normalization | `"hello"`, `"WORLD"`, `"  Rust  "`, `"\tTrIe\n"`, `"a"`, `"Z"` | `"HELLO"`, `"WORLD"`, `"RUST"`, `"TRIE"`, `"A"`, `"Z"` | Passed (0.00s) |
| **T08** | `test_normalize_empty_and_whitespace` | `trie/src/lib.rs` | Empty input rejection | `""`, `"   "`, `"\t\r\n"` | `Err(TrieError::EmptyWord)` | Passed (0.00s) |
| **T09** | `test_normalize_invalid_characters` | `trie/src/lib.rs` | Invalid character rejection | `"hello!"`, `"123"`, `"cat dog"`, `"café"`, `"word-trie"`, `"🦀"` | `Err(TrieError::InvalidCharacter(c))` | Passed (0.00s) |
| **T10** | `test_insert_single_word` | `trie/src/lib.rs` | Single word insertion | Insert `"APPLE"` | `Ok(true)`, `len() == 1`, `!is_empty()` | Passed (0.00s) |
| **T11** | `test_insert_multiple_unrelated_words` | `trie/src/lib.rs` | Multiple disjoint words | `"APPLE"`, `"BANANA"`, `"CHERRY"` | All `Ok(true)`, `len() == 3` | Passed (0.00s) |
| **T12** | `test_insert_shared_prefixes` | `trie/src/lib.rs` | Shared prefix insertion | `"CAR"`, `"CARD"`, `"CART"`, `"CARE"` | All `Ok(true)`, `len() == 4` | Passed (0.00s) |
| **T13** | `test_insert_prefix_of_existing_word` | `trie/src/lib.rs` | Inserting prefix after longer word | Insert `"CARD"`, then `"CAR"` | Both `Ok(true)`, `len() == 2` | Passed (0.00s) |
| **T14** | `test_insert_duplicate` | `trie/src/lib.rs` | Duplicate insertion | `"RUST"`, `"RUST"`, `"rust"`, `"  RUST  "` | First `Ok(true)`, others `Ok(false)`, `len() == 1` | Passed (0.00s) |
| **T15** | `test_insert_case_normalization` | `trie/src/lib.rs` | Case insensitivity in insert | `"orange"`, then `"OrAnGe"` | First `Ok(true)`, second `Ok(false)`, `len() == 1` | Passed (0.00s) |
| **T16** | `test_insert_invalid_input` | `trie/src/lib.rs` | Safe rejection during insert | `""`, `"word1"`, `"hello!"`, `"über"` | All `Err(TrieError)`, `len() == 0` | Passed (0.00s) |
| **T17** | `test_insert_short_and_long_words` | `trie/src/lib.rs` | Length boundaries | `"A"`, `"Z"`, `"SUPERCALIFRAGILISTICEXPIALIDOCIOUS"` | All `Ok(true)`, `len() == 3` | Passed (0.00s) |
| **T18** | `test_lookup_contains_existing_and_missing` | `trie/src/lib.rs` | Lookup exact matching | Insert `"RUST"`, `"TREE"`; Query `"RUST"`, `"RUS"`, `"RUSTY"`, `"PYTHON"` | `true`, `false`, `false`, `false` | Passed (0.00s) |
| **T19** | `test_lookup_case_normalization` | `trie/src/lib.rs` | Lookup case insensitivity | Insert `"Rust"`; Query `"rust"`, `"RUST"`, `"  RuSt  "` | All `true` | Passed (0.00s) |
| **T20** | `test_lookup_empty_and_invalid_returns_false` | `trie/src/lib.rs` | Invalid lookup queries | `""`, `"   "`, `"VALID!"`, `"VAL1D"` | All return `false` | Passed (0.00s) |
| **T21** | `test_has_prefix` | `trie/src/lib.rs` | Prefix existence checks | Insert `"CART"`, `"CAT"`; Query `"C"`, `"CA"`, `"CAR"`, `"CART"`, `"DOG"` | `true`, `true`, `true`, `true`, `false` | Passed (0.00s) |
| **T22** | `test_remove_leaf_word_and_pruning` | `trie/src/lib.rs` | Deleting leaf word and pruning | Insert `"CAR"`, `"CARD"`; Remove `"CARD"` | `Ok(true)`, `contains("CARD") == false`, `contains("CAR") == true`, `has_prefix("CARD") == false` | Passed (0.00s) |
| **T23** | `test_remove_prefix_word_preserves_longer_word` | `trie/src/lib.rs` | Deleting prefix word | Insert `"CAR"`, `"CARD"`; Remove `"CAR"` | `Ok(true)`, `contains("CAR") == false`, `contains("CARD") == true`, `has_prefix("CAR") == true` | Passed (0.00s) |
| **T24** | `test_remove_nonexistent_and_prefix_only` | `trie/src/lib.rs` | Deleting absent words | Insert `"CART"`; Remove `"DOG"`, Remove `"CAR"` | Both `Ok(false)`, `len() == 1`, `"CART"` preserved | Passed (0.00s) |
| **T25** | `test_remove_repeated` | `trie/src/lib.rs` | Repeated deletion | Insert `"TEST"`; Remove `"TEST"`; Remove `"TEST"` again | First `Ok(true)`, second `Ok(false)`, `len() == 0` | Passed (0.00s) |
| **T26** | `test_remove_until_empty` | `trie/src/lib.rs` | Deleting all words | Insert `"A"`, `"TO"`, `"TEA"`; Remove all | All `Ok(true)`, `len() == 0`, `is_empty() == true`, prefixes pruned | Passed (0.00s) |
| **T27** | `test_remove_case_normalization` | `trie/src/lib.rs` | Case insensitivity in deletion | Insert `"RUST"`; Remove `"rust"` | `Ok(true)`, `len() == 0` | Passed (0.00s) |
| **T28** | `test_remove_invalid_input` | `trie/src/lib.rs` | Error reporting on invalid delete | `""`, `"123"`, `"HELLO!"` | Returns `Err(TrieError)`, word count preserved | Passed (0.00s) |
| **T29** | `test_remove_words_batch` | `trie/src/lib.rs` | Batch word deletion | Insert `"ONE"`, `"TWO"`, `"THREE"`; Remove `["one", "four", "two"]` | `[Ok(true), Ok(false), Ok(true)]`, `len() == 1`, `"THREE"` preserved | Passed (0.00s) |
| **T30** | `test_differential_randomized_operations` | `trie/tests/differential_testing.rs` | Randomized stress test vs HashSet | 2,000 pseudo-random operations (insert, remove, lookup, prefix) | Exact agreement on return values, membership, and `len()` at every step | Passed (0.01s) |
| **T31** | `test_alphabet_boundary_extremities` | `trie/tests/differential_testing.rs` | Alphabet boundaries | `"A"`, `"Z"`, `"AAAA"`, `"ZZZZ"`, `"AM"`, `"ZM"` | All operations and pruning work at extremes | Passed (0.00s) |
| **T32** | `test_boundary_error_handling` | `trie/tests/differential_testing.rs` | Boundary inputs and errors | Mixed spaces, tabs, internal spaces, punctuation | Exact `TrieError` variants returned | Passed (0.00s) |
| **T33** | `test_parse_tokens_various_separators` | `trie-cli/src/batch.rs` | Delimited token parser | `"  apple, banana   cherry\n\n date, \tfig,,,grape  "` | `["apple", "banana", "cherry", "date", "fig", "grape"]` | Passed (0.00s) |
| **T34** | `test_parse_tokens_empty_and_whitespace` | `trie-cli/src/batch.rs` | Parser on empty / delimiter-only string | `""`, `"    \n\t,,\t, "` | Empty vector | Passed (0.00s) |
| **T35** | `test_load_words_from_reader_mixed_content` | `trie-cli/src/batch.rs` | Buffered line loading | `"apple\nbanana, cherry\n\n  APPLE  \n123, date!\n"` | 6 tokens, 3 new words, 1 duplicate, 2 errors with line numbers | Passed (0.00s) |
| **T36** | `test_load_empty_reader` | `trie-cli/src/batch.rs` | Loading from empty reader | Cursor on `""` | 0 tokens, 0 new words, `len() == 0` | Passed (0.00s) |
| **T37** | `test_check_words_from_reader` | `trie-cli/src/batch.rs` | Batch classification | Trie contains `"apple"`, `"banana"`; Candidates `"APPLE, orange\nBANANA, grape, bad#token\n"` | Found: `["APPLE", "BANANA"]`; Missing: `["ORANGE", "GRAPE"]`; Invalid: 1 | Passed (0.00s) |
| **T38** | `test_load_and_check_file_io` | `trie-cli/src/batch.rs` | Temp file I/O loading | Temp files with dictionary and candidate words | Words loaded and classified accurately from filesystem | Passed (0.01s) |
| **T39** | `test_file_not_found_error` | `trie-cli/src/batch.rs` | Missing file error handling | `"nonexistent_path_xyz123.txt"` | Returns `io::ErrorKind::NotFound` | Passed (0.00s) |
| **T40** | `test_process_command_help` | `trie-cli/src/main.rs` | CLI `help` command | `"help"` | Output contains usage for all commands | Passed (0.00s) |
| **T41** | `test_process_command_insert_find_delete` | `trie-cli/src/main.rs` | CLI core workflow | `"insert apple, banana"`, `"find apple, cherry"`, `"delete apple"` | Insert reported, find matches, delete reported | Passed (0.00s) |
| **T42** | `test_process_command_found_missing` | `trie-cli/src/main.rs` | CLI `found` and `missing` | `"found car, train, bus"`, `"missing car, train, bus, plane"` | Accurately lists found and missing sets | Passed (0.00s) |
| **T43** | `test_process_command_stats` | `trie-cli/src/main.rs` | CLI `stats` command | `"stats"` on empty and populated Trie | Reports count and status correctly | Passed (0.00s) |
| **T44** | `test_process_command_quit` | `trie-cli/src/main.rs` | CLI termination commands | `"quit"`, `"EXIT"` | Returns `Ok(false)` | Passed (0.00s) |
| **T45** | `test_process_command_unknown_and_empty` | `trie-cli/src/main.rs` | CLI edge inputs | `""`, `"   "`, `"foobar"` | Empty ignored, unknown command reported without panic | Passed (0.00s) |
| **T46** | `test_full_dictionary_loading_and_lookup` | `trie-cli/src/main.rs` | Full-dictionary validation | Loads `dictionary.txt` (176,413 words) | Matches reference HashSet count, queries known words, verifies prefix behavior | Passed (0.89s) |
| **D01–D08** | Doctests (8 methods) | `trie/src/lib.rs` | Runnable doc examples | Public API examples | All doctests compiled and passed | Passed (3.10s) |

---

## 3. Independent Reference Testing (Oracle Testing)

To verify the Trie against an authoritative baseline, [`trie/tests/differential_testing.rs`](file:///c:/Users/User/Downloads/Programming%20Lab/Word%20Games/word-trie/trie/tests/differential_testing.rs) implements randomized differential testing:

- **Reference Model**: Rust standard library `std::collections::HashSet<String>`.
- **PRNG**: A 64-bit Xorshift PRNG (`SimpleRng`) seeded with constant `0xDEAD_BEEF_CAFE_BABE`. This guarantees 100% deterministic, reproducible runs without external crate dependencies.
- **Alphabet**: Restricted alphabet `b"ABCDE"` to deliberately maximize prefix collisions, shared branches, and nested deletions.
- **Workflow (2,000 steps)**:
  - At each step, a pseudo-random operation (Insert, Remove, Lookup, Prefix Check) is chosen.
  - The operation is performed on both the `Trie` and the `HashSet`.
  - The return values are asserted equal.
  - The total count `trie.len() == reference.len()` is asserted equal after every single step.
  - Invariant `trie.is_empty() == reference.is_empty()` is asserted equal after every single step.
  - A final sweep verifies all words remaining in the `HashSet` are present in the `Trie`.

---

## 4. Full-Dictionary Smoke Test

The test `test_full_dictionary_loading_and_lookup` executes an end-to-end integration test on the supplied dictionary:
1. Opens `dictionary.txt` and performs an independent count using a `BufReader` and `HashSet<String>`.
2. Loads `dictionary.txt` into a new `Trie` using `load_words_from_file`.
3. Asserts:
   - `trie.len() == reference_set.len()` (176,413).
   - `report.newly_inserted == 176,413`.
   - `report.duplicates == 0`.
   - `report.invalid_entries.len() == 0`.
4. Performs lookups for representative words across the alphabet (`"AA"`, `"AAH"`, `"AARDVARK"`, `"CAT"`, `"DOG"`, `"ELEPHANT"`, `"MIDDLE"`, `"PROGRAMMING"`, `"RUST"`, `"TREE"`, `"ZYZZYVA"`, `"ZZZ"`).
5. Tests lowercase lookups (`"rust"` $\to$ found).
6. Tests non-existent lookups (`"NOTAWORD123"`, `"FOOBARXYZ"` $\to$ not found).
7. Tests prefix queries (`"PROG"` $\to$ `has_prefix == true`).
8. Mutates the full Trie (removes `"AA"`) and verifies that `len() == 176,412`, `"AA"` is absent, and neighbor `"AAH"` remains intact.
