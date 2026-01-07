//! The basic framework to build an API.

pub mod env;
pub mod framework;
pub mod shutdown;
pub mod transactions;
pub mod workflow;

/// A shorthand to define a statically allocated variable using a [`std::sync::LazyLock`].
///
/// # Examples
///
/// ```rust
/// static_lazy_lock!{
///     pub VAR_1: String = String::from("a static variable");
/// }
/// // ...equals to...
/// pub static VAR_2: LazyLock<String> = LazyLock::new(|| String::from("a static variable"));
/// ```
#[macro_export]
macro_rules! static_lazy_lock {
    ($(#[$meta:meta])* $vis:vis $name:ident: $type:ty = $expr:expr $(;)?) => {
        $(#[$meta])*
        $vis static $name: $crate::__priv_macro_use::LazyLock<$type> =
            $crate::__priv_macro_use::LazyLock::new(|| $expr);
    };
}

#[doc(hidden)]
pub mod __priv_macro_use {
    pub use std::sync::LazyLock;
}
