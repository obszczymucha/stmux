pub(crate) fn is_numeric(as_str: &str) -> bool {
    as_str.chars().all(char::is_numeric)
}
