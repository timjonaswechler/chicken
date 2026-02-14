#[macro_export]
macro_rules! int_pnt {
    ($x:expr, $y:expr) => {
        $crate::i_float::int::point::IntPoint::new($x, $y)
    };
}
