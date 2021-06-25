//! Defines useful internal macros

/// Macro to assert if two doubles are equal to a certain default precision of `f64::EPSILON`.
#[allow(unused_macros)]
macro_rules! assert_f64_eq {
    ($x:expr, $y:expr) => {
        if ($x - $y).abs() > std::f64::EPSILON {
            panic!("{} != {}", $x, $y);
        }
    };
    ($x:expr, $y:expr, $d:expr) => {
        if ($x - $y).abs() > $d {
            panic!("{} != {}", $x, $y);
        }
    };
}

/// Macro to assert if two iterable doubles are equal to a certain default precision of
/// `f64::EPSILON`.
#[allow(unused_macros)]
macro_rules! assert_f64_iter_eq {
    ($x:expr, $y:expr) => {
        for (a, b) in $x.iter().zip($y.iter()) {
            assert_f64_eq!(a, b);
        }
    };
    ($x:expr, $y:expr, $d:expr) => {
        for (a, b) in $x.iter().zip($y.iter()) {
            assert_f64_eq!(a, b, $d);
        }
    };
}
