use pebble::recommended_id_length;

#[test]
fn test_id_length_breakpoints() {
    // This test verifies the transition points where the recommended ID length increases.
    // The tuples represent (population_size, expected_id_length).
    // We test the exact boundary `N` where the length becomes `L`, and `N-1` where it was `L-1`.
    // The calculation aims to keep collision probability < 10^-12 based on the Birthday Paradox.
    let breakpoints = [
        (0, 8),
        (1, 8),
        (2, 9),
        (13, 9),
        (14, 10),
        (84, 10),
        (85, 11),
        (512, 11),
        (513, 12),
        (3077, 12),
        (3078, 13),
        (18469, 13),
        (18470, 14),
        (110822, 14),
        (110823, 15),
        (664940, 15),
        (664941, 16),
        (1_000_000, 16),
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
