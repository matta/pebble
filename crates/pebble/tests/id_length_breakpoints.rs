use pebble::recommended_id_length;

#[test]
fn test_id_length_breakpoints() {
    // Breakpoints calculated using the formula: N = sqrt(2 * P * 36^L)
    // where P = 1e-12.
    // However, the function uses:
    // required_pool_size = k^2 / (2 * P)
    // 36^L >= required_pool_size
    // k^2 <= 36^L * 2 * P
    // k <= sqrt(36^L * 2 * P)
    //
    // Let's verify exact thresholds.
    // ALPHABET_SIZE = 36.0
    // TARGET_PROBABILITY = 1.0e-12
    // 2 * P = 2.0e-12
    //
    // L=1: N <= sqrt(36^1 * 2e-12) = sqrt(72e-12) = sqrt(7.2e-11) approx 8.48e-6.
    // Wait, recommended_id_length(k) returns 1 if k <= 1.
    //
    // Let's re-run logic for small k.
    // k=0 -> 1
    // k=1 -> 1
    // k=2: req_pool = 4 / 2e-12 = 2e12. 36^L >= 2e12.
    // 36^7 = 78,364,164,096 < 2e12
    // 36^8 = 2,821,109,907,456 > 2e12.
    // So for k=2, length should be 8.
    //
    // Wait, my manual calc for k=1 in previous turn:
    // k=1 -> req_pool = 1 / 2e-12 = 5e11.
    // 36^7 = 7.8e10 < 5e11.
    // 36^8 = 2.8e12 > 5e11.
    // So for k=1 (if not capped), length should be 8.
    // BUT the function explicitly returns 1 for k<=1.
    //
    // So:
    // k=0 -> 1
    // k=1 -> 1
    // k=2 -> 8 (jump!)
    //
    // Let's check larger k for breakpoints.
    // We want to find the max k for a given length L.
    // k_max(L) = floor(sqrt(36^L * 2e-12))
    //
    // L=1: sqrt(36 * 2e-12) ~ small.
    // ...
    // L=7: sqrt(36^7 * 2e-12) = sqrt(7.8e10 * 2e-12) = sqrt(0.15) ~ 0.39.
    // L=8: sqrt(36^8 * 2e-12) = sqrt(2.8e12 * 2e-12) = sqrt(5.6) ~ 2.37 -> floor is 2.
    // So for k=2, length 8 is sufficient (barely).
    // For k=3, req = 9 / 2e-12 = 4.5e12. 36^8 = 2.8e12. So L=8 not enough.
    // 36^9 = 1e14. So L=9 needed.
    //
    // Let's generate the test data based on running the function.
    // I will write a test that finds the transition points by brute force or binary search?
    // Brute force is fast enough for small numbers.

    let breakpoints = [
        (0, 1),
        (1, 1),
        (2, 8),
        (3, 9), // k=3 -> req=4.5e12. 36^8=2.8e12 (fail). 36^9=1e14 (pass).
        // Let's verify k=2 transition.
        // k=2 -> req=2e12. 36^8=2.8e12. Pass. L=8.
        //
        // What about k where L increases to 10?
        // 36^9 = 1.01e14.
        // k_max(9) = sqrt(1.01e14 * 2e-12) = sqrt(202) ~ 14.2.
        // So up to k=14, L=9.
        // k=15 -> req = 225 / 2e-12 = 1.125e14. > 36^9. So L=10.
        (14, 9),
        (15, 10),
        // L=10 -> 36^10 = 3.65e15.
        // k_max(10) = sqrt(3.65e15 * 2e-12) = sqrt(7311) ~ 85.5.
        // So up to k=85, L=10.
        (85, 10),
        (86, 11),
        // L=11 -> 36^11 = 1.31e17.
        // k_max(11) = sqrt(1.31e17 * 2e-12) = sqrt(263209) ~ 513.
        (513, 11),
        (514, 12),
        // L=12 -> 36^12 = 4.73e18.
        // k_max(12) = sqrt(4.73e18 * 2e-12) = sqrt(9.47e6) ~ 3078.
        (3078, 12),
        (3079, 13),
    ];

    for (k, expected_len) in breakpoints {
        let len = recommended_id_length(k);
        assert_eq!(
            len, expected_len,
            "For population {}, expected length {}, got {}",
            k, expected_len, len
        );
    }
}
