use nanoid::nanoid;
use std::collections::HashSet;

const ID_ALPHABET: &[char; 36] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];
const MAX_ATTEMPTS_PER_LENGTH: usize = 1_000;

pub fn generate_unique_id(
    prefix: &str,
    existing_ids: &HashSet<String>,
    suffix_length: usize,
) -> String {
    // TODO(matt): Add a file lock to prevent concurrent add collisions.
    generate_unique_id_with(prefix, existing_ids, suffix_length, |len| {
        nanoid!(len, ID_ALPHABET)
    })
}

fn generate_unique_id_with<F>(
    prefix: &str,
    existing_ids: &HashSet<String>,
    mut suffix_length: usize,
    mut generate_suffix: F,
) -> String
where
    F: FnMut(usize) -> String,
{
    loop {
        for _ in 0..MAX_ATTEMPTS_PER_LENGTH {
            let suffix = generate_suffix(suffix_length);
            let id = format!("{prefix}-{suffix}");
            if !existing_ids.contains(&id) {
                return id;
            }
        }

        suffix_length += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_unique_id_first_try() {
        let existing_ids = HashSet::new();
        let id = generate_unique_id_with("issue", &existing_ids, 4, |_| "abcd".to_string());
        assert_eq!(id, "issue-abcd");
    }

    #[test]
    fn bumps_length_after_exhausting_attempts() {
        let mut existing_ids = HashSet::new();
        existing_ids.insert("issue-aaaa".to_string());

        let mut calls = 0;
        let id = generate_unique_id_with("issue", &existing_ids, 4, |len| {
            calls += 1;
            if len == 4 {
                "aaaa".to_string()
            } else {
                "bbbbb".to_string()
            }
        });

        let suffix = id
            .strip_prefix("issue-")
            .expect("id should start with issue-");
        assert_eq!(suffix.len(), 5);
        assert_eq!(suffix, "bbbbb");
        assert_eq!(calls, MAX_ATTEMPTS_PER_LENGTH + 1);
    }

    #[test]
    fn generated_suffix_uses_expected_alphabet() {
        let existing_ids = HashSet::new();
        let id = generate_unique_id("issue", &existing_ids, 16);
        let suffix = id
            .strip_prefix("issue-")
            .expect("id should start with issue-");

        assert_eq!(suffix.len(), 16);
        for ch in suffix.chars() {
            assert!(
                ID_ALPHABET.contains(&ch),
                "unexpected character in suffix: {ch}"
            );
        }
    }
}
