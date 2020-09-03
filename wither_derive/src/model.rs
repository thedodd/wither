use darling::FromMeta;
use inflector::Inflector;
use proc_macro_error::abort;
use quote::quote;
use syn::DeriveInput;

/// The name of the helper attribute used by this derive macro.
const MODEL_HELPER_ATTR: &str = "model";
/// An error message indicating the existence of a duplicate attr.
const DUPLICATE_ATTR_SPEC: &str = "duplicate attr specification";
/// An error message describing the correct form expected for an attribute.
const META_MUST_BE_KV_PAIR: &str = "this attribute must be specified as a `key=value` pair";

/// A meta representation of the `Model` derivation target.
pub(crate) struct MetaModel<'a> {
    ident: &'a syn::Ident,
    attrs: &'a [syn::Attribute],
    fields: Vec<FieldWithFilteredAttrs<'a>>,
    /// The model's collection name; will default to a formatted and pluralized form of the struct's
    /// name.
    collection_name: Option<String>,
    /// A flag to configure if serde checks should be skipped.
    skip_serde_checks: Option<()>,
    /// All indexes derived on this model.
    indexes: Vec<IndexModelTokens>,
    /// The model's read concern; will default to None if not specified.
    ///
    /// NOTE WELL: there is currently an issue with darling's parsing of enums where if the value
    /// is not specified in one of the ways which darling expects, it will behave unexpectedly. See
    /// https://github.com/TedDriggs/darling/issues/74#issuecomment-635761725
    pub read_concern: Option<ReadConcern>,
    /// The model's write concern; will default to None if not specified.
    pub write_concern: Option<WriteConcern>,
    /// The function which should be called to get the model's selection criteria; will default to
    /// None if not specified.
    pub selection_criteria: Option<syn::Path>,
}

impl<'a> MetaModel<'a> {
    /// Create a new instance.
    pub fn new(input: &'a DeriveInput) -> Self {
        // The target's ident.
        let ident = &input.ident;
        // Extract struct's named fields.
        let fields = match &input.data {
            syn::Data::Struct(struct_data) => match &struct_data.fields {
                syn::Fields::Named(named_fields) => named_fields,
                _ => abort!(&input, "wither models must have named fields"),
            },
            _ => abort!(&input, "only structs can be used as wither models"),
        };
        let mut inst = Self {
            ident,
            attrs: input.attrs.as_slice(),
            fields: vec![],
            indexes: vec![],
            collection_name: None,
            skip_serde_checks: None,
            read_concern: None,
            write_concern: None,
            selection_criteria: None,
        };

        // Parse attrs for struct-level model attrs.
        inst.extract_model_attrs();
        // Extract the fields of this model & filter down to pertinent attrs per field.
        inst.extract_model_fields(fields);
        // Validate the model's ID field.
        inst.check_id_field();
        inst
    }

    /// Expand the model into the full model impl output.
    pub fn expand(&self) -> proc_macro2::TokenStream {
        let name = self.ident;
        let collection_name = self.get_collection_name();
        let read_concern = OptionReadConcern(&self.read_concern);
        let write_concern = OptionWriteConcern(&self.write_concern);
        let selection_criteria = OptionSelectionCriteria(&self.selection_criteria);
        let indexes = &self.indexes;
        quote! {
            #[wither::async_trait]
            impl wither::Model for #name {
                const COLLECTION_NAME: &'static str = #collection_name;

                /// Get a cloned copy of this instance's ID.
                fn id(&self) -> ::std::option::Option<wither::bson::oid::ObjectId> {
                    self.id.clone()
                }

                /// Set this instance's ID.
                fn set_id(&mut self, oid: wither::bson::oid::ObjectId) {
                    self.id = Some(oid);
                }

                /// The model's read concern.
                fn read_concern() -> Option<wither::mongodb::options::ReadConcern> {
                    #read_concern
                }

                /// The model's write concern.
                fn write_concern() -> Option<wither::mongodb::options::WriteConcern> {
                    #write_concern
                }

                /// The model's selection criteria.
                ///
                /// When deriving a model, a function or an associated function should be specified which
                /// should be used to produce the desired value.
                fn selection_criteria() -> Option<wither::mongodb::options::SelectionCriteria> {
                    #selection_criteria
                }

                /// All indexes currently on this model.
                fn indexes() -> Vec<wither::IndexModel> {
                    vec![#(#indexes),*]
                }
            }
        }
    }

    // NOTE WELL: this is pending removal per https://github.com/thedodd/wither/issues/52
    // /// Expand the model into the full sync model impl output.
    // pub fn expand_sync(&self) -> proc_macro2::TokenStream {
    //     let name = self.ident;
    //     let collection_name = self.get_collection_name();
    //     let read_concern = OptionReadConcern(&self.read_concern);
    //     let write_concern = OptionWriteConcern(&self.write_concern);
    //     let selection_criteria = OptionSelectionCriteria(&self.selection_criteria);
    //     let indexes = &self.indexes;
    //     quote! {
    //         impl wither::ModelSync for #name {
    //             const COLLECTION_NAME: &'static str = #collection_name;

    //             /// Get a cloned copy of this instance's ID.
    //             fn id(&self) -> ::std::option::Option<wither::bson::oid::ObjectId> {
    //                 self.id.clone()
    //             }

    //             /// Set this instance's ID.
    //             fn set_id(&mut self, oid: wither::bson::oid::ObjectId) {
    //                 self.id = Some(oid);
    //             }

    //             /// The model's read concern.
    //             fn read_concern() -> Option<wither::mongodb::options::ReadConcern> {
    //                 #read_concern
    //             }

    //             /// The model's write concern.
    //             fn write_concern() -> Option<wither::mongodb::options::WriteConcern> {
    //                 #write_concern
    //             }

    //             /// The model's selection criteria.
    //             ///
    //             /// When deriving a model, a function or an associated function should be specified
    // which             /// should be used to produce the desired value.
    //             fn selection_criteria() -> Option<wither::mongodb::options::SelectionCriteria> {
    //                 #selection_criteria
    //             }

    //             /// All indexes currently on this model.
    //             fn indexes() -> Vec<wither::IndexModel> {
    //                 vec![#(#indexes),*]
    //             }
    //         }
    //     }
    // }

    /// Extract any model attrs and bind them to their optional slots.
    fn extract_model_attrs(&mut self) {
        let attrs = Self::parse_attrs(&self.attrs, MODEL_HELPER_ATTR);
        // Parse over the internals of our `model` attrs. At this point, we are dealing with
        // individual elements inside of the various `model(...)` attrs.
        for attr_meta in attrs {
            let ident = attr_meta
                .path()
                .get_ident()
                .unwrap_or_else(|| abort!(attr_meta, "malformed wither model attribute, please review the wither docs"));
            let ident_str = ident.to_string();
            match ident_str.as_str() {
                "collection_name" => self.extract_collection_name(&attr_meta),
                "index" => self.extract_index(&attr_meta),
                "read_concern" => self.extract_read_concern(&attr_meta),
                "selection_criteria" => self.extract_selection_criteria(&attr_meta),
                "skip_serde_checks" => self.extract_skip_serde_checks(&attr_meta),
                "write_concern" => self.extract_write_concern(&attr_meta),
                _ => abort!(ident, "unrecognized wither model attribute"),
            }
        }
    }

    /// Extract the collection name from the given meta.
    fn extract_collection_name(&mut self, meta: &syn::Meta) {
        let name = match meta {
            syn::Meta::NameValue(val) => match &val.lit {
                syn::Lit::Str(inner) => inner.value(),
                lit => abort!(lit, "this must be a string literal"),
            },
            _ => abort!(meta, META_MUST_BE_KV_PAIR),
        };
        if name.is_empty() {
            abort!(meta, "wither model collection names must be at least one character in length");
        }
        if self.collection_name.is_some() {
            abort!(meta, DUPLICATE_ATTR_SPEC);
        }
        self.collection_name = Some(name);
    }

    /// Extract an index attribute from the given meta.
    fn extract_index(&mut self, meta: &syn::Meta) {
        let idx = match RawIndexModel::from_meta(meta) {
            Ok(idx) => idx,
            Err(err) => abort!(meta, "malformed wither model index specification"; hint=err),
        };
        self.indexes.push(IndexModelTokens::from(idx));
    }

    /// Extract the read concern attribute from the given meta.
    fn extract_read_concern(&mut self, meta: &syn::Meta) {
        let rc = match ReadConcern::from_meta(meta) {
            Ok(rc) => rc,
            Err(err) => abort!(meta, "malformed wither model read concern attribute"; hint=err),
        };
        if self.read_concern.is_some() {
            abort!(meta, DUPLICATE_ATTR_SPEC);
        }
        self.read_concern = Some(rc);
    }

    /// Extract the selection criteria attribute from the given meta.
    fn extract_selection_criteria(&mut self, meta: &syn::Meta) {
        let fnpath = match meta {
            syn::Meta::NameValue(val) => match syn::Path::from_value(&val.lit) {
                Ok(path) => path,
                Err(err) => abort!(val, "this must be a string literal"; hint=err),
            },
            _ => abort!(meta, META_MUST_BE_KV_PAIR),
        };
        if self.selection_criteria.is_some() {
            abort!(meta, DUPLICATE_ATTR_SPEC);
        }
        self.selection_criteria = Some(fnpath);
    }

    /// Extract the skip serde checks attribute from the given meta.
    fn extract_skip_serde_checks(&mut self, meta: &syn::Meta) {
        match meta {
            syn::Meta::Path(path) if path.is_ident("skip_serde_checks") => (),
            _ => abort!(meta, "this attribute must be specified simply as `#[model(skip_serde_checks)]`"),
        }
        if self.skip_serde_checks.is_some() {
            abort!(meta, DUPLICATE_ATTR_SPEC);
        }
        self.skip_serde_checks = Some(());
    }

    /// Extract the write concern attribute from the given meta.
    fn extract_write_concern(&mut self, meta: &syn::Meta) {
        let wc = match WriteConcern::from_meta(meta) {
            Ok(wc) => wc,
            Err(err) => abort!(meta, "malformed wither model write concern attribute"; hint=err),
        };
        if self.write_concern.is_some() {
            abort!(meta, DUPLICATE_ATTR_SPEC);
        }
        self.write_concern = Some(wc);
    }

    /// Extract the indexes on this model.
    fn extract_model_fields(&mut self, fields: &'a syn::FieldsNamed) {
        self.fields = fields.named.iter()
            // Build an IR of the fields which holds the original field object & its filtered attrs.
            .map(|field| {
                let serde_attrs = Self::parse_attrs(&field.attrs, "serde");
                FieldWithFilteredAttrs{serde_attrs, field}
            })
            .collect();
    }

    /// Get collection name which is to be used for this model.
    fn get_collection_name(&self) -> String {
        self.collection_name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.ident.to_string().to_table_case().to_plural())
    }

    /// Parse the given slice of attrs and return an accumulation of each individual attr within the
    /// parent `#[model(...)]` list.
    fn parse_attrs(attrs: &[syn::Attribute], container_name: &str) -> Vec<syn::Meta> {
        attrs.iter()
            // Only process attrs matching the given container name.
            .filter(|attr| attr.path.is_ident(container_name))
            // Only process valid meta attrs.
            .filter_map(|attr| match attr.parse_meta() {
                Ok(meta) => Some(meta),
                Err(err) => abort!(attr, "malformed attribute"; hint=err),
            })
            // Extract the inner meta list of the target attrs.
            .map(|meta| match meta {
                syn::Meta::List(inner) => inner.nested,
                _ => abort!(meta, format!("wither expected this attribute to be formatted as a meta list, eg: `#[{}(...)]`", container_name)),
            })
            // Accumulate all attrs so that we can deal with them as a single iterable.
            .fold(vec![], |mut acc, nested| {
                for inner in nested {
                    match inner {
                        syn::NestedMeta::Meta(meta) => acc.push(meta),
                        syn::NestedMeta::Lit(lit) => abort!(lit, "unexpected literal value"),
                    }
                }
                acc
            })
    }

    /// Ensure the model has an ID field which is structured as needed.
    ///
    /// NB: the type of the ID field is not checked here. The compiler still checks that the type
    /// matches as needed when the AST is written back out to the compiler.
    fn check_id_field(&self) {
        // Unpack the struct fields.
        // Look for the model's ID field.
        let id_field = self
            .fields
            .iter()
            .find(|field| match &field.field.ident {
                Some(ident) => ident == "id",
                None => false,
            })
            .unwrap_or_else(|| abort!(self.ident, "wither models must have a field `id` of type `Option<bson::oid::ObjectId>`"));
        // Ensure the ID field has needed serde attributes, unless this check is disabled.
        if self.skip_serde_checks.is_none() {
            self.check_id_serde_attrs(id_field);
        }
    }

    // Ensure the `id` field has required serde attrs.
    fn check_id_serde_attrs(&self, id_field: &FieldWithFilteredAttrs<'a>) {
        let mut found_rename = false;
        let mut found_skip = false;
        for attr in &id_field.serde_attrs {
            if attr.path().is_ident("rename") {
                let model = SerdeIdRename::from_meta(attr).unwrap_or_else(|err| abort!(attr, "failed to parse serde rename attr"; hint=err));
                if model.0 != "_id" {
                    abort!(attr, r#"the serde `rename` attr for wither::Model ID fields should be `rename="_id"`"#);
                }
                found_rename = true;
            }
            if attr.path().is_ident("skip_serializing_if") {
                let model =
                    SerdeIdSkip::from_meta(attr).unwrap_or_else(|err| abort!(attr, "failed to parse serde skip_serializing_if attr"; hint=err));
                if model.0 != "Option::is_none" {
                    abort!(
                        attr,
                        r#"the serde `skip_serializing_if` attr for wither::Model ID fields should be `skip_serializing_if="Option::is_none"`"#
                    );
                }
                found_skip = true
            }
            if found_rename && found_skip {
                break;
            }
        }
        // If no serde attrs were found on the ID field, display error with expections.
        if !(found_rename && found_skip) {
            abort!(
                id_field.field.ident,
                r#"the ID field of wither::Models must have the attribute `#[serde(rename="_id", skip_serializing_if="Option::is_none")]`"#
            )
        }
    }
}

/// A model field with a set of filtered model & serde attrs on that field.
pub struct FieldWithFilteredAttrs<'a> {
    /// All collected serde attributes.
    serde_attrs: Vec<syn::Meta>,
    /// The original field.
    field: &'a syn::Field,
}

/// A required serde attr for renaming the ID field during serialization & deserialization.
#[derive(FromMeta)]
pub struct SerdeIdRename(pub String);

/// A required serde attr for skipping serialization of the `id` field is null.
#[derive(FromMeta)]
pub struct SerdeIdSkip(pub String);

//////////////////////////////////////////////////////////////////////////////
// ReadConcern ///////////////////////////////////////////////////////////////

/// A type wrapper around the `mongodb::options::ReadConcern` type.
#[derive(FromMeta)]
pub enum ReadConcern {
    Local,
    Majority,
    Linearizable,
    Available,
    Custom(String),
}

/// A wrapper around an optional read concern.
pub struct OptionReadConcern<'a>(&'a Option<ReadConcern>);

impl quote::ToTokens for OptionReadConcern<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self.0 {
            None => tokens.extend(quote!(None)),
            Some(ReadConcern::Local) => tokens.extend(quote!(Some(wither::mongodb::options::ReadConcern::local()))),
            Some(ReadConcern::Majority) => tokens.extend(quote!(Some(wither::mongodb::options::ReadConcern::majority()))),
            Some(ReadConcern::Linearizable) => tokens.extend(quote!(Some(wither::mongodb::options::ReadConcern::linearizable()))),
            Some(ReadConcern::Available) => tokens.extend(quote!(Some(wither::mongodb::options::ReadConcern::available()))),
            Some(ReadConcern::Custom(val)) => tokens.extend(quote!(Some(wither::mongodb::options::ReadConcern::custom(String::from(#val))))),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////
// WriteConcern //////////////////////////////////////////////////////////////

/// A type wrapper around the `mongodb::options::WriteConcern` type.
#[derive(FromMeta)]
pub struct WriteConcern {
    #[darling(default)]
    pub w: Option<Acknowledgment>,
    #[darling(default)]
    pub w_timeout: Option<u64>,
    #[darling(default)]
    pub journal: Option<bool>,
}

/// A type wrapper around the `mongodb::options::Acknowledgment` type.
#[derive(FromMeta)]
pub enum Acknowledgment {
    Nodes(i32),
    Majority,
    Custom(String),
}

pub struct OptionWriteConcern<'a>(&'a Option<WriteConcern>);

impl quote::ToTokens for OptionWriteConcern<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self.0 {
            None => tokens.extend(quote!(None)),
            Some(wc) => {
                let w = match &wc.w {
                    Some(ack) => match ack {
                        Acknowledgment::Nodes(val) => quote!(Some(wither::mongodb::options::Acknowledgment::Nodes(#val))),
                        Acknowledgment::Majority => quote!(Some(wither::mongodb::options::Acknowledgment::Majority)),
                        Acknowledgment::Custom(val) => quote!(Some(wither::mongodb::options::Acknowledgment::Custom(String::from(#val)))),
                    },
                    None => quote!(None),
                };
                let w_timeout = match &wc.w_timeout {
                    Some(val) => quote!(Some(::std::time::Duration::from_secs(#val))),
                    None => quote!(None),
                };
                let journal = match &wc.journal {
                    Some(val) => quote!(Some(#val)),
                    None => quote!(None),
                };
                tokens.extend(quote!(Some(wither::mongodb::options::WriteConcern::builder().w(#w).w_timeout(#w_timeout).journal(#journal).build())));
            }
        }
    }
}

//////////////////////////////////////////////////////////////////////////////
// SelectionCriteria /////////////////////////////////////////////////////////

pub struct OptionSelectionCriteria<'a>(&'a Option<syn::Path>);

impl quote::ToTokens for OptionSelectionCriteria<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self.0 {
            None => tokens.extend(quote!(None)),
            Some(path) => tokens.extend(quote!(Some(#path()))),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////
// Index Models //////////////////////////////////////////////////////////////

/// The raw model used for deriving indices on models.
#[derive(Debug, FromMeta)]
pub struct RawIndexModel {
    /// The document to use for the index keys.
    pub keys: darling::util::SpannedValue<String>,
    /// The document to use for the index options.
    #[darling(default)]
    pub options: darling::util::SpannedValue<Option<String>>,
}

impl From<RawIndexModel> for IndexModelTokens {
    fn from(src: RawIndexModel) -> Self {
        let keys = syn::parse_str(&src.keys).unwrap_or_else(|err| abort!(src.keys.span(), "error parsing keys, must be valid Rust code"; hint=err));
        let options = src.options.as_ref().as_ref().map(|opts| {
            syn::parse_str(opts.as_ref()).unwrap_or_else(|err| abort!(src.options.span(), "error parsing options, must be valid Rust code"; hint=err))
        });
        Self { keys, options }
    }
}

/// The set of token streams to use for building an index model.
pub struct IndexModelTokens {
    /// The token stream to use as an index model's keys.
    pub keys: proc_macro2::TokenStream,
    /// The token stream to use as an index model's options.
    pub options: Option<proc_macro2::TokenStream>,
}

impl quote::ToTokens for IndexModelTokens {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let options = match &self.options {
            Some(opts) => quote!(Some(#opts)),
            None => quote!(None),
        };
        let keys = &self.keys;
        tokens.extend(quote!(wither::IndexModel::new(#keys, #options)));
    }
}
