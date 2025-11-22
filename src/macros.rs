// Future & signal macros for easier async signal handling
#[macro_export]
macro_rules! future {
    (($($params:tt)*) $body:block) => {
        move |$($params)*| {
            $body

            async {}
        }
    };
}

#[macro_export]
macro_rules! signal {
    ($source:expr, ($($params:tt)*) $body:block) => {
        $source.signal().for_each($crate::future!(($($params)*) $body))
    };
}

#[macro_export]
macro_rules! signal_cloned {
    ($source:expr, ($($params:tt)*) $body:block) => {
        $source.signal_cloned().for_each($crate::future!(($($params)*) $body))
    };
}

#[macro_export]
macro_rules! signal_vec_cloned {
    ($source:expr, ($($params:tt)*) $body:block) => {
        $source.signal_vec_cloned().for_each($crate::future!(($($params)*) $body))
    };
}