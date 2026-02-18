use pebble::recommended_id_length;

#[test]
fn test_id_length_breakpoints() {
    // This test verifies the transition points where the recommended ID length increases.
    // The tuples represent (population_size, expected_id_length).
    // We test the exact boundary `N` where the length becomes `L`, and `N-1` where it was `L-1`.
    // The calculation aims to keep collision probability < 10^-12 based on the Birthday Paradox.
    let breakpoints = [
        (0, 1),
        (1, 1),
        (2, 8),
        (3, 9),
        (14, 9),
        (15, 10),
        (85, 10),
        (86, 11),
        (513, 11),
        (514, 12),
        (3078, 12),
        (3079, 13),
    ];

    for (population, expected_len) in breakpoints {
        let len = recommended_id_length(population);
        assert_eq!(
            len, expected_len,
            "For population {}, expected length {}, got {}",
            population, expected_len, len
        );
    }
}
