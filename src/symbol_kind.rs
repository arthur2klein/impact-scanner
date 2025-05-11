use std::slice::Iter;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
/// Kind of symbols to care about in source files.
pub enum SymbolKind {
    /// Includes every named callable.
    Function,
}
impl SymbolKind {
    /// Iterates over every element of the `SymbolKind`enum.
    ///
    /// ## Returns:
    /// - (`Iter<'static, SymbolKind>`): Iterator over all elements of the enum.
    pub fn iter() -> Iter<'static, SymbolKind> {
        [SymbolKind::Function].iter()
    }
}
