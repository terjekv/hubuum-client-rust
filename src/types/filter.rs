// FilterOperator enum
#[derive(Debug, Clone)]
pub enum FilterOperator {
    Eq,
    NotEq,
    Gt,
    Lt,
    GtEq,
    LtEq,
    Contains,
    StartsWith,
    EndsWith,
}
