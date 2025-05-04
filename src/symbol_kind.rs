use std::slice::Iter;

#[derive(PartialEq, Eq, Debug)]
pub enum SymbolKind {
    Function,
}
impl SymbolKind {
    pub fn iter() -> Iter<'static, SymbolKind> {
        [SymbolKind::Function].iter()
    }
}
