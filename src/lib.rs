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
    ($vis:vis $name:ident: $type:ty = $expr:expr; $($doc:expr)?) => {
        $(#[doc=$doc])?
        $vis static $name: std::sync::LazyLock<$type> =
            std::sync::LazyLock::new(|| $expr);
    };
}
