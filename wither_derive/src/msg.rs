/// An error message indicating the serde field attribute requirements on the `Model` `id` field.
pub(crate) const ID_FIELD_SERDE_REQ: &str = r#"A `Model` struct must have a field named `id`, and it must have the following attribute: `#[serde(rename="_id", skip_serializing_if="Option::is_none")]`."#;

/// An error message indicating the required form of `Model` attributes.
pub(crate) const MODEL_ATTR_FORM: &str = "All `Model` attributes must take the form `#[model(...)]`.";

/// An error message indicating the required form of a `#[model(index(...))]` attribute.
pub(crate) const MODEL_ATTR_INDEX_FORM: &str = "The `model(index)` attribute must have its own set of values, as such: `#[model(index(...))]`.";

/// An error message indicating the required form of elements within an index declaration.
pub(crate) const MODEL_ATTR_INDEX_ELEMENT_FORM: &str = "Index declarations on your model fields may only contain name-value pairs or the nested `with(...)` element.";

/// An error message indicating the allowed values for `#[model(index(direction))]`.
pub(crate) const MODEL_ATTR_INDEX_DIRECTION_ALLOWED_VALUES: &str = r#"The index `direction` value must be one of `"asc"` or `"dsc"`."#;

/// An error message indicating the allowed form for `#[model(index(with(...)))]`.
pub(crate) const MODEL_ATTR_INDEX_WITH_FORM: &str = r#"The `model(index(with(...))) attr may contain only mappings of field names to directions (`"asc"` or `"dsc"`)."#;

/// An error message indicating the required form of a `Model`'s struct level attributes.
pub(crate) const MODEL_STRUCT_ATTR_FORM: &str = r#"A `Model`'s struct level attributes may only contain name-value pairs: `#[model(name="value")]`."#;
