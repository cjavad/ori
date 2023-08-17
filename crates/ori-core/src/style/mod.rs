pub mod builtin;
mod key;
mod palette;
mod theme;

pub use key::*;
pub use palette::*;
pub use theme::*;

#[macro_export]
macro_rules! style {
    (
        $(#[$module_attr:meta])*
        $module_vis:vis $module:ident {
            $(
                $(#[$attr:meta])*
                const $name:ident : $ty:ty = $expr:expr;
            )*
        }
    ) => {
        $(#[$module_attr])*
        $module_vis mod $module {
            use super::*;

            $(
                $(#[$attr])*
                pub const $name: $crate::Key<$ty> = $crate::Key::new(
                    ::std::concat!(::std::stringify!($module), ".", ::std::stringify!($name))
                );
            )*

            /// Get the default theme for this module.
            pub fn default_theme() -> $crate::Theme {
                $crate::Theme::new()
                    $(.with($name, $expr))*
            }
        }
    };
}
