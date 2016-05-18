// Audio Utilities ------------------------------------------------------------
const MAX_SEQ_NUMBER: u16 = 65535;

pub fn seq_is_more_recent(a: u16, b: u16) -> bool {
    (a > b) && (a - b <= MAX_SEQ_NUMBER / 2) ||
    (b > a) && (b - a >  MAX_SEQ_NUMBER / 2)
}

pub fn compress(x: f32, threshold: f32) -> f32 {

    // threshold    alpha
    // ------------------
    // 0            2.51
    // 0.05         2.67
    // 0.1          2.84
    // 0.15         3.04
    // 0.2          3.26
    // 0.25         3.52
    // 0.3          3.82
    // 0.35         4.17
    // 0.4          4.59
    // 0.45         5.09
    // 0.5          5.71
    // 0.55         6.49
    // 0.6          7.48
    // 0.65         8.81
    // 0.7          10.63
    // 0.75         13.3
    // 0.8          17.51
    // 0.85         24.97
    // 0.9          41.15
    // 0.95         96.09
    if x >= -threshold && x <= threshold {
        x

    } else {
        let alpha = 7.48; // for threshold=0.6
        let xa = x.abs();
        let a = (1.0 + alpha * ((xa - threshold) / (2.0 - threshold))).ln();
        let b = (1.0 + alpha).ln();
        (x / xa) * (threshold + (1.0 - threshold) * (a / b))
    }

}

