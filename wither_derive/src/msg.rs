/// An error message indicating the serde field attribute requirements on the `Model` `id` field.
pub(crate) const ID_FIELD_SERDE_REQ: &str = r#"A `Model` struct must have a field named `id`, and it must have the following attribute: `#[serde(rename="_id", skip_serializing_if="Option::is_none")]`."#;

/// An error message indicating the required form of `Model` attributes.
pub(crate) const MODEL_ATTR_FORM: &str = "All `Model` attributes must take the form `#[model(...)]`.";

/// An error message indicating the required form of a `#[model(index(...))]` attribute.
pub(crate) const MODEL_ATTR_INDEX_FORM: &str = "The `model(index)` attribute must have its own set of values, as such: `#[model(index(...))]`.";

/// An error message indicating the allowed values for `#[model(index(direction))]`.
pub(crate) const MODEL_ATTR_INDEX_TYPE_ALLOWED_VALUES: &str = r#"The value for `index="..."` must be one of `"asc"`, `"dsc"`, `"2d"`, `"2dsphere"`, `"geoHaystack"`, `"text"`, or `"hashed"`."#;

/// An error message indicating the allowed form for `#[model(index(weights(...)))]`.
pub(crate) const MODEL_ATTR_INDEX_WEIGHTS_FORM: &str = r#"The `model(index(weight(...))) attr must be of form `weight(field="...", weight="...")`, where `field` is the field name, and `weight` is an `i32` wrapped in a string."#;

/// An error message indicating the allowed form for `#[model(index(with(...)))]`.
pub(crate) const MODEL_ATTR_INDEX_WITH_FORM: &str = r#"The `model(index(with(...))) attr must be of form `with(field="...", index="...")` where `field="..."` is the field to include and `index="..."` is one of the valid index types (`"asc"`, `"dsc"`, `"2d"`, `"2dsphere"`, `"geoHaystack"`, `"text"`, `"hashed"`)."#;

/// An error message indicating the allowed form for `#[model(index(embedded="..."))]`.
pub(crate) const MODEL_ATTR_INDEX_EMBEDDED_FORM: &str = r#"The `model(index(embedded)) attr must be of the form `embedded="..."`."#;
