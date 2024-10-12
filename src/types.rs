// #todo Find a better name than Dialect, maybe Flavor?
// #todo Could we make dialects/formatting pluggable?

/// The dialect of the source Tan. The formatter offer customized formatting for
/// different Dialects.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Dialect {
    Code,
    Data,
    Html,
    Css,
}

impl Default for Dialect {
    fn default() -> Self {
        Self::Code
    }
}
