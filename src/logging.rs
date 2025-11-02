// TODO: These are placehoders for now

#[macro_export]
macro_rules! info {
    ($( $tt:tt )*) => {
        $crate::nvim::print!($( $tt )*)
    }
}

#[macro_export]
macro_rules! warning {
    ($( $tt:tt )*) => {
        $crate::nvim::print!($( $tt )*)
    }
}

#[macro_export]
macro_rules! error {
    ($( $tt:tt )*) => {
        $crate::nvim::print!($( $tt )*)
    }
}
