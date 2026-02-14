//! Proc macro crate for chicken_settings derive macros.
//!
//! This crate provides derive macros for:
//! - `#[derive(Settings)]` - Implements the `Settings` trait (and `StaticSettings` or `DynamicSettings` based on context)
//! - `#[derive(SettingsContext)]` - Implements the `PathContext` trait

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Expr, Lit, Type};

/// Derive macro for the `Settings` trait.
///
/// Automatically implements:
/// - `Settings` trait (always)
/// - `StaticSettings` marker trait (when no context is specified)
/// - `DynamicSettings` trait (when `context = Type` is specified)
///
/// # Attributes
///
/// - `#[settings(path = "path/to/file.toml")]` - Static path with format auto-detection
/// - `#[settings(path = "path/{field}.toml", context = ContextType)]` - Dynamic path with placeholders
///
/// # Examples
///
/// ```ignore
/// // Static settings (no context)
/// #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone)]
/// #[settings(path = "settings/audio.toml")]
/// struct AudioSettings {
///     volume: f32,
/// }
///
/// // Dynamic settings (with context)
/// #[derive(SettingsContext, Resource, Clone)]
/// struct SaveContext {
///     slot_id: u32,
/// }
///
/// #[derive(Settings, Resource, Serialize, Deserialize, Default, Clone)]
/// #[settings(path = "saves/{slot_id}/player.toml", context = SaveContext)]
/// struct PlayerSettings {
///     name: String,
/// }
/// ```
#[proc_macro_derive(Settings, attributes(settings))]
pub fn derive_settings(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    // Parse the settings attribute
    let settings_attr = match parse_settings_attribute(&input.attrs) {
        Ok(Some(attr)) => attr,
        Ok(None) => {
            return syn::Error::new_spanned(
                input.ident,
                "Missing #[settings(path = \"path\")] attribute",
            )
            .to_compile_error()
            .into();
        }
        Err(e) => return e.to_compile_error().into(),
    };

    let path_template = &settings_attr.path;
    let has_dynamic_path = path_template.contains('{') && path_template.contains('}');
    let has_context = settings_attr.context.is_some();
    let context_type = settings_attr
        .context
        .unwrap_or_else(|| syn::parse_quote!(()));

    // Generate the Settings trait implementation (always)
    let settings_impl = quote! {
        impl #impl_generics chicken_settings::Settings for #name #type_generics #where_clause {
            type Context = #context_type;

            fn type_name() -> &'static str {
                stringify!(#name)
            }

            fn path_template() -> &'static str {
                #path_template
            }

            fn has_dynamic_path() -> bool {
                #has_dynamic_path
            }

            fn format() -> chicken_settings::Format {
                chicken_settings::Format::from_path(#path_template)
                    .unwrap_or(chicken_settings::Format::Toml)
            }

            fn into_box(self) -> Box<dyn std::any::Any + Send + Sync> {
                Box::new(self)
            }

            fn clone_box(&self) -> Box<dyn std::any::Any + Send + Sync> {
                Box::new(self.clone())
            }
        }
    };

    // Generate additional trait implementation based on whether context is present
    let extra_impl = if has_context {
        // Dynamic settings: implement DynamicSettings trait
        quote! {
            impl #impl_generics chicken_settings::DynamicSettings for #name #type_generics #where_clause {
                fn resolve_path(context: &Self::Context) -> chicken_settings::path::SettingsPath {
                    context.resolve_path(#path_template)
                        .expect("Failed to resolve dynamic path")
                }
            }
        }
    } else {
        // Static settings: implement StaticSettings marker trait
        quote! {
            impl #impl_generics chicken_settings::StaticSettings for #name #type_generics #where_clause {}
        }
    };

    TokenStream::from(quote! {
        #settings_impl
        #extra_impl
    })
}

/// Parsed settings attribute data
struct SettingsAttribute {
    path: String,
    context: Option<Type>,
}

fn parse_settings_attribute(attrs: &[Attribute]) -> Result<Option<SettingsAttribute>, syn::Error> {
    for attr in attrs {
        if attr.path().is_ident("settings") {
            let mut path: Option<String> = None;
            let mut context: Option<Type> = None;

            // Try parsing as nested meta: #[settings(path = "...", context = Type)]
            let nested_result = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("path") {
                    meta.input.parse::<syn::Token![=]>()?;
                    let expr: Expr = meta.input.parse()?;
                    if let Expr::Lit(expr_lit) = expr {
                        if let Lit::Str(lit_str) = expr_lit.lit {
                            path = Some(lit_str.value());
                        }
                    }
                    Ok(())
                } else if meta.path.is_ident("context") {
                    meta.input.parse::<syn::Token![=]>()?;
                    let expr: Expr = meta.input.parse()?;
                    if let Expr::Path(expr_path) = expr {
                        context = Some(syn::parse2(quote!(#expr_path)).unwrap());
                    }
                    Ok(())
                } else {
                    Ok(())
                }
            });

            // If nested meta parsing failed, it might be legacy syntax
            if let Err(e) = nested_result {
                // Only report error if we can't parse as legacy syntax either
                // Store the error for potential later use
                let nested_error = e;

                // Try legacy syntax: #[settings("path")] or #[settings(path = "...", context = Type)]
                attr.parse_args_with(|input: syn::parse::ParseStream| {
                    // Try to parse as: "path" or path = "..."
                    if input.peek(syn::LitStr) {
                        let lit: syn::LitStr = input.parse()?;
                        path = Some(lit.value());
                    } else {
                        let ident: syn::Ident = input.parse()?;
                        if ident == "path" {
                            input.parse::<syn::Token![=]>()?;
                            let lit: syn::LitStr = input.parse()?;
                            path = Some(lit.value());
                        }
                    }

                    // Check for comma and context
                    if input.parse::<syn::Token![,]>().is_ok() {
                        let ident: syn::Ident = input.parse()?;
                        if ident == "context" {
                            input.parse::<syn::Token![=]>()?;
                            let expr: Expr = input.parse()?;
                            if let Expr::Path(expr_path) = expr {
                                context = Some(syn::parse2(quote!(#expr_path)).unwrap());
                            }
                        }
                    }
                    Ok(())
                })
                .map_err(|e| syn::Error::new(
                    attr.span(),
                    format!("Invalid #[settings] attribute syntax: {}. Expected format: #[settings(path = \"...\")] or #[settings(path = \"...\", context = Type)]", e)
                ))?;

                // If we still don't have a path, report the original nested meta error
                if path.is_none() {
                    return Err(syn::Error::new(
                        attr.span(),
                        format!("Invalid #[settings] attribute syntax: {}. Expected format: #[settings(path = \"...\")] or #[settings(path = \"...\", context = Type)]", nested_error)
                    ));
                }
            }

            // If nested meta didn't set path, try legacy syntax
            if path.is_none() {
                attr.parse_args_with(|input: syn::parse::ParseStream| {
                    // Try to parse as: "path" or path = "..."
                    if input.peek(syn::LitStr) {
                        let lit: syn::LitStr = input.parse()?;
                        path = Some(lit.value());
                    } else {
                        let ident: syn::Ident = input.parse()?;
                        if ident == "path" {
                            input.parse::<syn::Token![=]>()?;
                            let lit: syn::LitStr = input.parse()?;
                            path = Some(lit.value());
                        }
                    }

                    // Check for comma and context
                    if input.parse::<syn::Token![,]>().is_ok() {
                        let ident: syn::Ident = input.parse()?;
                        if ident == "context" {
                            input.parse::<syn::Token![=]>()?;
                            let expr: Expr = input.parse()?;
                            if let Expr::Path(expr_path) = expr {
                                context = Some(syn::parse2(quote!(#expr_path)).unwrap());
                            }
                        }
                    }
                    Ok(())
                })
                .map_err(|e| syn::Error::new(
                    attr.span(),
                    format!("Invalid #[settings] attribute syntax in legacy format: {}. Expected: #[settings(\"path/to/file.toml\")] or #[settings(path = \"...\", context = Type)]", e)
                ))?;
            }

            // Ensure we have a path
            let path = path.ok_or_else(|| syn::Error::new(
                attr.span(),
                "Missing `path` in #[settings] attribute. Expected format: #[settings(path = \"...\")]"
            ))?;

            return Ok(Some(SettingsAttribute { path, context }));
        }
    }
    Ok(None)
}

/// Derive macro for the `PathContext` trait.
///
/// Automatically generates `resolve_placeholder()` and `to_map()` methods
/// based on the struct fields.
///
/// # Example
///
/// ```ignore
/// #[derive(SettingsContext, Resource, Clone)]
/// struct SaveContext {
///     slot_id: u32,
///     player_name: String,
/// }
/// ```
///
/// This generates:
/// ```ignore
/// impl PathContext for SaveContext {
///     fn resolve_placeholder(&self, name: &str) -> Option<String> {
///         match name {
///             "slot_id" => Some(self.slot_id.to_string()),
///             "player_name" => Some(self.player_name.to_string()),
///             _ => None,
///         }
///     }
///
///     fn to_map(&self) -> HashMap<String, String> {
///         let mut map = HashMap::new();
///         map.insert("slot_id".to_string(), self.slot_id.to_string());
///         map.insert("player_name".to_string(), self.player_name.to_string());
///         map
///     }
/// }
/// ```
#[proc_macro_derive(SettingsContext)]
pub fn derive_settings_context(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    // Extract field names
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => {
            return syn::Error::new_spanned(
                input.ident,
                "SettingsContext can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate match arms for resolve_placeholder
    let match_arms: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            let field_name = f.ident.as_ref()?;
            let field_name_str = field_name.to_string();
            Some(quote! {
                #field_name_str => Some(self.#field_name.to_string()),
            })
        })
        .collect();

    // Generate map insertions for to_map
    let map_inserts: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            let field_name = f.ident.as_ref()?;
            let field_name_str = field_name.to_string();
            Some(quote! {
                map.insert(#field_name_str.to_string(), self.#field_name.to_string());
            })
        })
        .collect();

    let expanded = quote! {
        impl #impl_generics chicken_settings::path::PathContext for #name #type_generics #where_clause {
            fn resolve_placeholder(&self, name: &str) -> Option<String> {
                match name {
                    #(#match_arms)*
                    _ => None,
                }
            }

            fn to_map(&self) -> std::collections::HashMap<String, String> {
                let mut map = std::collections::HashMap::new();
                #(#map_inserts)*
                map
            }
        }
    };

    TokenStream::from(expanded)
}
