pub mod click;
pub mod dispatch;
pub mod registry;

pub(crate) use click::{ui_click_release_confirms, UiClickPress};
pub use registry::UiClickAction;
