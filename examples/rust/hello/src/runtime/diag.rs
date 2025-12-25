#[macro_export]
macro_rules! here {
    () => {
        concat!(file!(), ":", line!())
    };
}

#[macro_export]
macro_rules! callee {
    () => {
        callee!(__)
    };
    ( $name:ident ) => {{
        struct $name;
        std::any::type_name::<$name>()
    }};
}
