//! # Trie Library
//!
//! A 26-way prefix tree (Trie) dictionary implementation in Rust.
//!
//! Stores English words containing ASCII letters A–Z. Words are stored along paths
//! where common prefixes share the same sequence of nodes.

use std::fmt;

/// The size of the English alphabet (A through Z).
pub const ALPHABET_SIZE: usize = 26;

/// Error conditions produced during Trie operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrieError {
    /// The provided word is empty or contained only whitespace.
    EmptyWord,
    /// An unsupported or non-ASCII-alphabetic character was encountered.
    InvalidCharacter(char),
}

impl fmt::Display for TrieError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrieError::EmptyWord => write!(f, "word must not be empty"),
            TrieError::InvalidCharacter(c) => {
                write!(
                    f,
                    "invalid character '{c}'; only ASCII letters A-Z are allowed"
                )
            }
        }
    }
}

impl std::error::Error for TrieError {}

/// An internal node in the 26-way Trie.
///
/// Each node contains:
/// - `children`: A fixed-size array of 26 optional heap-allocated child nodes.
/// - `is_terminal`: A flag indicating whether the path from the root ending at
///   this node constitutes a complete, stored word.
#[derive(Debug)]
pub(crate) struct Node {
    pub(crate) children: [Option<Box<Node>>; ALPHABET_SIZE],
    pub(crate) is_terminal: bool,
}

impl Node {
    /// Creates a new, empty node with no children and `is_terminal = false`.
    pub(crate) fn new() -> Self {
        const NONE_CHILD: Option<Box<Node>> = None;
        Self {
            children: [NONE_CHILD; ALPHABET_SIZE],
            is_terminal: false,
        }
    }

    /// Checks if this node has at least one child.
    pub(crate) fn has_any_children(&self) -> bool {
        self.children.iter().any(|c| c.is_some())
    }

    /// A node is prunable if it is not a terminal word and has no remaining children.
    pub(crate) fn is_prunable(&self) -> bool {
        !self.is_terminal && !self.has_any_children()
    }

    /// Recursively removes a word path from this node.
    ///
    /// Returns `true` if a complete word was found and unmarked.
    /// Any dangling child node that becomes non-terminal and childless is pruned.
    pub(crate) fn remove_recursive(node: &mut Node, bytes: &[u8], depth: usize) -> bool {
        if depth == bytes.len() {
            if node.is_terminal {
                node.is_terminal = false;
                true
            } else {
                false
            }
        } else {
            let idx = (bytes[depth] - b'A') as usize;
            if let Some(child) = node.children[idx].as_deref_mut() {
                let removed = Self::remove_recursive(child, bytes, depth + 1);
                if removed && node.children[idx].as_ref().unwrap().is_prunable() {
                    node.children[idx] = None;
                }
                removed
            } else {
                false
            }
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalizes a candidate word by trimming whitespace and converting ASCII letters to uppercase.
///
/// Returns `Ok(String)` with all uppercase ASCII characters `A`–`Z`.
/// Returns `Err(TrieError::EmptyWord)` if the trimmed input is empty.
/// Returns `Err(TrieError::InvalidCharacter)` on the first non-ASCII-alphabetic character.
pub fn normalize_word(input: &str) -> Result<String, TrieError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(TrieError::EmptyWord);
    }

    let mut normalized = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if ch.is_ascii_alphabetic() {
            normalized.push(ch.to_ascii_uppercase());
        } else {
            return Err(TrieError::InvalidCharacter(ch));
        }
    }

    Ok(normalized)
}

/// Converts an uppercase ASCII character ('A'..='Z') to its 0-based alphabet index (0..=25).
///
/// Returns `None` if `ch` is outside the range `'A'..='Z'`.
#[inline]
pub fn char_to_index(ch: char) -> Option<usize> {
    if ch.is_ascii_uppercase() {
        Some((ch as u8 - b'A') as usize)
    } else {
        None
    }
}

/// Converts a 0-based alphabet index (0..=25) to its uppercase ASCII character ('A'..='Z').
///
/// Returns `None` if `index >= 26`.
#[inline]
pub fn index_to_char(index: usize) -> Option<char> {
    if index < ALPHABET_SIZE {
        Some((b'A' + index as u8) as char)
    } else {
        None
    }
}

/// A 26-way prefix tree (Trie) dictionary.
///
/// Stores distinct words composed of ASCII letters A through Z.
/// Common prefixes share tree nodes to minimize redundant storage.
#[derive(Debug)]
pub struct Trie {
    pub(crate) root: Node,
    pub(crate) word_count: usize,
}

impl Trie {
    /// Creates a new, empty `Trie`.
    ///
    /// # Examples
    ///
    /// ```
    /// use trie::Trie;
    ///
    /// let t = Trie::new();
    /// assert_eq!(t.len(), 0);
    /// assert!(t.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            root: Node::new(),
            word_count: 0,
        }
    }

    /// Returns the number of distinct complete words stored in the trie.
    ///
    /// This returns the word count in $O(1)$ time, distinct from the total number of nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use trie::Trie;
    ///
    /// let t = Trie::new();
    /// assert_eq!(t.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.word_count
    }

    /// Returns `true` if the trie contains no words.
    ///
    /// # Examples
    ///
    /// ```
    /// use trie::Trie;
    ///
    /// let t = Trie::new();
    /// assert!(t.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.word_count == 0
    }

    /// Inserts a word into the trie.
    ///
    /// The word is normalized by trimming whitespace and converting ASCII letters
    /// to uppercase. If the word contains invalid characters or is empty, an error
    /// is returned and the trie is not modified.
    ///
    /// Returns `Ok(true)` if the word was newly inserted, or `Ok(false)` if the
    /// word was already present in the trie.
    ///
    /// # Examples
    ///
    /// ```
    /// use trie::Trie;
    ///
    /// let mut trie = Trie::new();
    /// assert_eq!(trie.insert("rust"), Ok(true));
    /// assert_eq!(trie.insert("RUST"), Ok(false)); // Duplicate
    /// assert_eq!(trie.len(), 1);
    /// ```
    pub fn insert(&mut self, word: &str) -> Result<bool, TrieError> {
        let normalized = normalize_word(word)?;
        let mut current = &mut self.root;

        for ch in normalized.chars() {
            let idx = char_to_index(ch).expect("normalized char must be valid A-Z");
            current = current.children[idx].get_or_insert_with(|| Box::new(Node::new()));
        }

        if current.is_terminal {
            Ok(false)
        } else {
            current.is_terminal = true;
            self.word_count += 1;
            Ok(true)
        }
    }

    /// Internal helper that navigates to the node corresponding to a normalized string path.
    ///
    /// Returns `None` if the input is invalid/empty or if the path does not exist in the trie.
    fn find_node(&self, text: &str) -> Option<&Node> {
        let normalized = normalize_word(text).ok()?;
        let mut current = &self.root;

        for ch in normalized.chars() {
            let idx = char_to_index(ch)?;
            current = current.children[idx].as_deref()?;
        }

        Some(current)
    }

    /// Checks whether the trie contains the specified complete word.
    ///
    /// Input is normalized by trimming whitespace and converting ASCII characters
    /// to uppercase. If the word is empty, contains invalid characters, or is not
    /// in the trie, this method returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use trie::Trie;
    ///
    /// let mut trie = Trie::new();
    /// trie.insert("rust").unwrap();
    ///
    /// assert!(trie.contains("RUST"));
    /// assert!(trie.contains("rust"));
    /// assert!(!trie.contains("ru")); // Prefix only, not a complete word
    /// assert!(!trie.contains("ruby"));
    /// assert!(!trie.contains("")); // Empty word returns false
    /// ```
    pub fn contains(&self, word: &str) -> bool {
        self.find_node(word).is_some_and(|node| node.is_terminal)
    }

    /// Checks whether any stored word starts with the specified prefix.
    ///
    /// Input is normalized by trimming whitespace and converting ASCII characters
    /// to uppercase. Returns `true` if a nonempty valid prefix has a path in the
    /// trie, even if that prefix is not marked as a complete stored word.
    ///
    /// Returns `false` if the prefix does not exist, is empty, or contains invalid
    /// characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use trie::Trie;
    ///
    /// let mut trie = Trie::new();
    /// trie.insert("catalog").unwrap();
    ///
    /// assert!(trie.has_prefix("cat"));
    /// assert!(trie.has_prefix("CATALOG"));
    /// assert!(!trie.has_prefix("dog"));
    /// assert!(!trie.has_prefix("")); // Empty prefix returns false
    /// ```
    pub fn has_prefix(&self, prefix: &str) -> bool {
        self.find_node(prefix).is_some()
    }

    /// Removes a word from the trie and prunes any childless, non-terminal ancestor nodes.
    ///
    /// The word is normalized by trimming whitespace and converting ASCII letters
    /// to uppercase.
    ///
    /// - Returns `Ok(true)` if the word was found, marked as non-terminal, and
    ///   the distinct word count was decremented.
    /// - Returns `Ok(false)` if the word was not present in the trie (or only
    ///   existed as an incomplete prefix).
    /// - Returns `Err(TrieError)` if the input word is empty or contains
    ///   invalid characters.
    ///
    /// Existing words that share a prefix with the removed word are preserved.
    ///
    /// # Examples
    ///
    /// ```
    /// use trie::Trie;
    ///
    /// let mut trie = Trie::new();
    /// trie.insert("card").unwrap();
    /// trie.insert("car").unwrap();
    ///
    /// assert_eq!(trie.remove("car"), Ok(true));
    /// assert_eq!(trie.len(), 1);
    /// assert!(!trie.contains("car"));
    /// assert!(trie.contains("card")); // Longer word sharing prefix is preserved!
    ///
    /// assert_eq!(trie.remove("car"), Ok(false)); // Already removed
    /// ```
    pub fn remove(&mut self, word: &str) -> Result<bool, TrieError> {
        let normalized = normalize_word(word)?;
        let removed = Node::remove_recursive(&mut self.root, normalized.as_bytes(), 0);
        if removed {
            self.word_count -= 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Removes a list of words from the trie.
    ///
    /// For each word in the provided iterator, attempts removal and collects the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use trie::Trie;
    ///
    /// let mut trie = Trie::new();
    /// trie.insert("apple").unwrap();
    /// trie.insert("banana").unwrap();
    ///
    /// let results = trie.remove_words(["apple", "cherry", "banana"]);
    /// assert_eq!(results, vec![Ok(true), Ok(false), Ok(true)]);
    /// assert_eq!(trie.len(), 0);
    /// ```
    pub fn remove_words<I, S>(&mut self, words: I) -> Vec<Result<bool, TrieError>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        words.into_iter().map(|w| self.remove(w.as_ref())).collect()
    }

    /// Returns a read-only cursor positioned at the root node of the Trie.
    ///
    /// Holding the cursor borrows the `Trie` immutably, ensuring no concurrent
    /// mutations can occur while traversing.
    ///
    /// # Examples
    ///
    /// ```
    /// use trie::Trie;
    ///
    /// let mut trie = Trie::new();
    /// trie.insert("CAR").unwrap();
    ///
    /// let cursor = trie.root_cursor();
    /// assert!(!cursor.is_word());
    /// let c = cursor.advance('C').unwrap();
    /// assert!(!c.is_word());
    /// ```
    pub fn root_cursor(&self) -> TrieCursor<'_> {
        TrieCursor { node: &self.root }
    }
}

/// An immutable, zero-allocation cursor representing a position in the Trie.
///
/// Encapsulates Trie node traversal without exposing the internal `Node` structure
/// or child array storage. Allows step-by-step traversal along permitted character paths.
///
/// Implements `Copy` and `Clone` because it wraps an immutable reference `&'a Node`.
/// Copying the cursor simply copies a pointer, making recursive algorithms (such as
/// Spelling Bee solution generation and Boggle depth-first search) clean and idiomatic.
#[derive(Debug, Clone, Copy)]
pub struct TrieCursor<'a> {
    node: &'a Node,
}

impl<'a> TrieCursor<'a> {
    /// Advances the cursor to the child node corresponding to the given letter.
    ///
    /// Supports ASCII alphabetic characters (case-insensitive, normalized to uppercase).
    /// Returns `Some(TrieCursor)` pointing to the child node if the branch exists,
    /// or `None` if the branch does not exist or `letter` is not an ASCII alphabetic character.
    ///
    /// # Complexity
    ///
    /// Runs in $O(1)$ time with zero heap allocations.
    #[inline]
    pub fn advance(&self, letter: char) -> Option<TrieCursor<'a>> {
        let idx = char_to_index(letter.to_ascii_uppercase())?;
        self.node.children[idx]
            .as_deref()
            .map(|child| TrieCursor { node: child })
    }

    /// Returns `true` if the node at this cursor position represents the end of a complete word.
    ///
    /// # Complexity
    ///
    /// Runs in $O(1)$ time.
    #[inline]
    pub fn is_word(&self) -> bool {
        self.node.is_terminal
    }

    /// Returns `true` if the node at this cursor position has at least one outgoing branch.
    ///
    /// # Complexity
    ///
    /// Runs in $O(1)$ time (checking 26 optional pointers).
    #[inline]
    pub fn has_children(&self) -> bool {
        self.node.has_any_children()
    }
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_trie_is_empty() {
        let trie = Trie::new();
        assert_eq!(trie.len(), 0);
        assert!(trie.is_empty());
    }

    #[test]
    fn test_default_trie_is_empty() {
        let trie = Trie::default();
        assert_eq!(trie.len(), 0);
        assert!(trie.is_empty());
    }

    #[test]
    fn test_node_initial_state() {
        let node = Node::new();
        assert!(!node.is_terminal);
        for child in &node.children {
            assert!(child.is_none());
        }
    }

    #[test]
    fn test_normalize_valid_words() {
        assert_eq!(normalize_word("hello").unwrap(), "HELLO");
        assert_eq!(normalize_word("WORLD").unwrap(), "WORLD");
        assert_eq!(normalize_word("  Rust  ").unwrap(), "RUST");
        assert_eq!(normalize_word("\tTrIe\n").unwrap(), "TRIE");
        assert_eq!(normalize_word("a").unwrap(), "A");
        assert_eq!(normalize_word("Z").unwrap(), "Z");
    }

    #[test]
    fn test_normalize_empty_and_whitespace() {
        assert_eq!(normalize_word(""), Err(TrieError::EmptyWord));
        assert_eq!(normalize_word("   "), Err(TrieError::EmptyWord));
        assert_eq!(normalize_word("\t\r\n"), Err(TrieError::EmptyWord));
    }

    #[test]
    fn test_normalize_invalid_characters() {
        assert_eq!(
            normalize_word("hello!"),
            Err(TrieError::InvalidCharacter('!'))
        );
        assert_eq!(normalize_word("123"), Err(TrieError::InvalidCharacter('1')));
        assert_eq!(
            normalize_word("cat dog"),
            Err(TrieError::InvalidCharacter(' '))
        );
        assert_eq!(
            normalize_word("café"),
            Err(TrieError::InvalidCharacter('é'))
        );
        assert_eq!(
            normalize_word("word-trie"),
            Err(TrieError::InvalidCharacter('-'))
        );
        assert_eq!(normalize_word("🦀"), Err(TrieError::InvalidCharacter('🦀')));
    }

    #[test]
    fn test_char_to_index_mapping() {
        assert_eq!(char_to_index('A'), Some(0));
        assert_eq!(char_to_index('B'), Some(1));
        assert_eq!(char_to_index('Y'), Some(24));
        assert_eq!(char_to_index('Z'), Some(25));
        assert_eq!(char_to_index('a'), None);
        assert_eq!(char_to_index('1'), None);
        assert_eq!(char_to_index('!'), None);
    }

    #[test]
    fn test_index_to_char_mapping() {
        assert_eq!(index_to_char(0), Some('A'));
        assert_eq!(index_to_char(1), Some('B'));
        assert_eq!(index_to_char(24), Some('Y'));
        assert_eq!(index_to_char(25), Some('Z'));
        assert_eq!(index_to_char(26), None);
        assert_eq!(index_to_char(100), None);
    }

    #[test]
    fn test_round_trip_index_char() {
        for idx in 0..ALPHABET_SIZE {
            let ch = index_to_char(idx).expect("valid index must yield char");
            assert_eq!(char_to_index(ch), Some(idx));
        }
    }

    #[test]
    fn test_insert_single_word() {
        let mut trie = Trie::new();
        assert_eq!(trie.insert("APPLE"), Ok(true));
        assert_eq!(trie.len(), 1);
        assert!(!trie.is_empty());
    }

    #[test]
    fn test_insert_multiple_unrelated_words() {
        let mut trie = Trie::new();
        assert_eq!(trie.insert("APPLE"), Ok(true));
        assert_eq!(trie.insert("BANANA"), Ok(true));
        assert_eq!(trie.insert("CHERRY"), Ok(true));
        assert_eq!(trie.len(), 3);
    }

    #[test]
    fn test_insert_shared_prefixes() {
        let mut trie = Trie::new();
        assert_eq!(trie.insert("CAR"), Ok(true));
        assert_eq!(trie.insert("CARD"), Ok(true));
        assert_eq!(trie.insert("CART"), Ok(true));
        assert_eq!(trie.insert("CARE"), Ok(true));
        assert_eq!(trie.len(), 4);
    }

    #[test]
    fn test_insert_prefix_of_existing_word() {
        let mut trie = Trie::new();
        assert_eq!(trie.insert("CARD"), Ok(true));
        assert_eq!(trie.len(), 1);
        assert_eq!(trie.insert("CAR"), Ok(true));
        assert_eq!(trie.len(), 2);
    }

    #[test]
    fn test_insert_duplicate() {
        let mut trie = Trie::new();
        assert_eq!(trie.insert("RUST"), Ok(true));
        assert_eq!(trie.insert("RUST"), Ok(false));
        assert_eq!(trie.insert("rust"), Ok(false));
        assert_eq!(trie.insert("  RUST  "), Ok(false));
        assert_eq!(trie.len(), 1);
    }

    #[test]
    fn test_insert_case_normalization() {
        let mut trie = Trie::new();
        assert_eq!(trie.insert("orange"), Ok(true));
        assert_eq!(trie.len(), 1);
        assert_eq!(trie.insert("OrAnGe"), Ok(false));
        assert_eq!(trie.len(), 1);
    }

    #[test]
    fn test_insert_invalid_input() {
        let mut trie = Trie::new();
        assert_eq!(trie.insert(""), Err(TrieError::EmptyWord));
        assert_eq!(trie.insert("   "), Err(TrieError::EmptyWord));
        assert_eq!(trie.insert("word1"), Err(TrieError::InvalidCharacter('1')));
        assert_eq!(trie.insert("hello!"), Err(TrieError::InvalidCharacter('!')));
        assert_eq!(trie.insert("über"), Err(TrieError::InvalidCharacter('ü')));
        assert_eq!(trie.len(), 0);
        assert!(trie.is_empty());
    }

    #[test]
    fn test_insert_short_and_long_words() {
        let mut trie = Trie::new();
        assert_eq!(trie.insert("A"), Ok(true));
        assert_eq!(trie.insert("Z"), Ok(true));
        assert_eq!(trie.insert("SUPERCALIFRAGILISTICEXPIALIDOCIOUS"), Ok(true));
        assert_eq!(trie.len(), 3);
    }

    #[test]
    fn test_lookup_contains_existing_and_missing() {
        let mut trie = Trie::new();
        trie.insert("RUST").unwrap();
        trie.insert("TREE").unwrap();

        assert!(trie.contains("RUST"));
        assert!(trie.contains("TREE"));
        assert!(!trie.contains("RUS")); // prefix only
        assert!(!trie.contains("RUSTY")); // extension
        assert!(!trie.contains("TR"));
        assert!(!trie.contains("PYTHON"));
    }

    #[test]
    fn test_lookup_case_normalization() {
        let mut trie = Trie::new();
        trie.insert("Rust").unwrap();

        assert!(trie.contains("rust"));
        assert!(trie.contains("RUST"));
        assert!(trie.contains("  RuSt  "));
    }

    #[test]
    fn test_lookup_empty_and_invalid_returns_false() {
        let mut trie = Trie::new();
        trie.insert("VALID").unwrap();

        assert!(!trie.contains(""));
        assert!(!trie.contains("   "));
        assert!(!trie.contains("VALID!"));
        assert!(!trie.contains("VAL1D"));
    }

    #[test]
    fn test_has_prefix() {
        let mut trie = Trie::new();
        trie.insert("CART").unwrap();
        trie.insert("CAT").unwrap();

        // Incomplete prefixes
        assert!(trie.has_prefix("C"));
        assert!(trie.has_prefix("CA"));
        assert!(trie.has_prefix("CAR"));

        // Complete words as prefixes
        assert!(trie.has_prefix("CART"));
        assert!(trie.has_prefix("CAT"));

        // Case insensitivity
        assert!(trie.has_prefix("ca"));
        assert!(trie.has_prefix("  cart "));

        // Non-existent prefixes
        assert!(!trie.has_prefix("DOG"));
        assert!(!trie.has_prefix("CARD"));
        assert!(!trie.has_prefix("CARTS"));

        // Invalid / empty
        assert!(!trie.has_prefix(""));
        assert!(!trie.has_prefix("   "));
        assert!(!trie.has_prefix("CA!"));
    }

    #[test]
    fn test_remove_leaf_word_and_pruning() {
        let mut trie = Trie::new();
        trie.insert("CAR").unwrap();
        trie.insert("CARD").unwrap();
        assert_eq!(trie.len(), 2);

        // Remove leaf word "CARD"
        assert_eq!(trie.remove("CARD"), Ok(true));
        assert_eq!(trie.len(), 1);
        assert!(!trie.contains("CARD"));
        assert!(trie.contains("CAR"));

        // Pruning check: path "CARD" no longer exists as a prefix
        assert!(!trie.has_prefix("CARD"));
        assert!(trie.has_prefix("CAR"));
    }

    #[test]
    fn test_remove_prefix_word_preserves_longer_word() {
        let mut trie = Trie::new();
        trie.insert("CAR").unwrap();
        trie.insert("CARD").unwrap();

        // Remove prefix word "CAR"
        assert_eq!(trie.remove("CAR"), Ok(true));
        assert_eq!(trie.len(), 1);
        assert!(!trie.contains("CAR"));
        assert!(trie.contains("CARD"));

        // Prefix path "CAR" still exists because "CARD" uses it
        assert!(trie.has_prefix("CAR"));
        assert!(trie.has_prefix("CARD"));
    }

    #[test]
    fn test_remove_nonexistent_and_prefix_only() {
        let mut trie = Trie::new();
        trie.insert("CART").unwrap();

        // Absent word
        assert_eq!(trie.remove("DOG"), Ok(false));
        // Prefix of an existing word, but not itself terminal
        assert_eq!(trie.remove("CAR"), Ok(false));
        assert_eq!(trie.len(), 1);
        assert!(trie.contains("CART"));
    }

    #[test]
    fn test_remove_repeated() {
        let mut trie = Trie::new();
        trie.insert("TEST").unwrap();

        assert_eq!(trie.remove("TEST"), Ok(true));
        assert_eq!(trie.len(), 0);
        assert_eq!(trie.remove("TEST"), Ok(false));
        assert_eq!(trie.len(), 0);
    }

    #[test]
    fn test_remove_until_empty() {
        let mut trie = Trie::new();
        trie.insert("A").unwrap();
        trie.insert("TO").unwrap();
        trie.insert("TEA").unwrap();
        assert_eq!(trie.len(), 3);

        assert_eq!(trie.remove("TEA"), Ok(true));
        assert_eq!(trie.remove("TO"), Ok(true));
        assert_eq!(trie.remove("A"), Ok(true));

        assert_eq!(trie.len(), 0);
        assert!(trie.is_empty());
        assert!(!trie.has_prefix("T"));
        assert!(!trie.has_prefix("A"));
    }

    #[test]
    fn test_remove_case_normalization() {
        let mut trie = Trie::new();
        trie.insert("RUST").unwrap();

        assert_eq!(trie.remove("rust"), Ok(true));
        assert_eq!(trie.len(), 0);
        assert!(!trie.contains("RUST"));
    }

    #[test]
    fn test_remove_invalid_input() {
        let mut trie = Trie::new();
        trie.insert("HELLO").unwrap();

        assert_eq!(trie.remove(""), Err(TrieError::EmptyWord));
        assert_eq!(trie.remove("   "), Err(TrieError::EmptyWord));
        assert_eq!(trie.remove("123"), Err(TrieError::InvalidCharacter('1')));
        assert_eq!(trie.remove("HELLO!"), Err(TrieError::InvalidCharacter('!')));
        assert_eq!(trie.len(), 1);
    }

    #[test]
    fn test_remove_words_batch() {
        let mut trie = Trie::new();
        trie.insert("ONE").unwrap();
        trie.insert("TWO").unwrap();
        trie.insert("THREE").unwrap();

        let results = trie.remove_words(["one", "four", "two"]);
        assert_eq!(results, vec![Ok(true), Ok(false), Ok(true)]);
        assert_eq!(trie.len(), 1);
        assert!(trie.contains("THREE"));
    }

    #[test]
    fn test_cursor_root_empty_trie() {
        let trie = Trie::new();
        let cursor = trie.root_cursor();
        assert!(!cursor.is_word());
        assert!(!cursor.has_children());
        assert!(cursor.advance('A').is_none());
    }

    #[test]
    fn test_cursor_advance_existing_prefix_and_absent() {
        let mut trie = Trie::new();
        trie.insert("CAR").unwrap();
        trie.insert("CART").unwrap();

        let root = trie.root_cursor();
        assert!(!root.is_word());
        assert!(root.has_children());

        // Absent branch
        assert!(root.advance('D').is_none());
        assert!(root.advance('1').is_none());
        assert!(root.advance('!').is_none());

        // Advance through 'C'
        let c = root.advance('C').expect("C branch exists");
        assert!(!c.is_word());
        assert!(c.has_children());

        // Advance through 'A' (supports case-insensitivity)
        let ca = c.advance('a').expect("A branch exists");
        assert!(!ca.is_word());
        assert!(ca.has_children());

        // Advance through 'R' -> Complete word "CAR"
        let car = ca.advance('R').expect("R branch exists");
        assert!(car.is_word(), "'CAR' is a complete word");
        assert!(car.has_children(), "'CAR' has further extensions ('T')");

        // Continuing from complete word to longer word: 'T' -> "CART"
        let cart = car.advance('T').expect("T branch exists");
        assert!(cart.is_word(), "'CART' is a complete word");
        assert!(!cart.has_children(), "'CART' has no further children");

        // Attempting to advance past leaf returns None
        assert!(cart.advance('S').is_none());
    }

    #[test]
    fn test_cursor_copy_and_branching() {
        let mut trie = Trie::new();
        trie.insert("BAT").unwrap();
        trie.insert("BAR").unwrap();

        let root = trie.root_cursor();
        let ba = root.advance('B').unwrap().advance('A').unwrap();

        // Copy cursor into two independent branches
        let branch_t = ba.advance('T').unwrap();
        let branch_r = ba.advance('R').unwrap();

        assert!(branch_t.is_word());
        assert!(branch_r.is_word());
        // Verify ba itself was not mutated
        assert!(!ba.is_word());
    }
}
