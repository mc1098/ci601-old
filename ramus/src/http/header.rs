enum HeaderFieldName {
    Registered(StandardFieldName),
    Custom(String),
}

macro_rules! standard_field_name_impl {
    (
        $variant:ident, $static_ident:ident, $name:literal
    ) => {};
}
