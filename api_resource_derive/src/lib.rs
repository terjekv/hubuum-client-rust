// #[derive(ApiResource)]
// pub struct Class {
//     #[api(read_only)]
//     pub id: i32,
//     pub name: String,
//     pub description: String,
//     pub namespace_id: i32,
//     pub json_schema: Option<serde_json::Value>,
//     pub validate_schema: Option<bool>,
//     #[api(read_only)]
//     pub created_at: chrono::NaiveDateTime,
//     #[api(read_only)]
//     pub updated_at: chrono::NaiveDateTime,
// }
// The endpoint becomes GetClass.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta};

use syn::punctuated::Punctuated;
use syn::token::Comma;

fn pluralize(name: &syn::Ident) -> String {
    let name = name.to_string();
    let last_char = name.chars().last().unwrap();
    match last_char {
        's' => format!("{}es", name),
        _ => format!("{}s", name),
    }
}

#[proc_macro_derive(ApiResource, attributes(endpoint, api))]
pub fn api_resource_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let base_name = name.to_string();
    if !base_name.ends_with("Resource") {
        panic!("ApiResource only supports structs with names ending in 'Resource'");
    }
    let name = format_ident!("{}", base_name.trim_end_matches("Resource"));
    let plural_name = format_ident!("{}", pluralize(&name));

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("ApiResource only supports structs with named fields"),
        },
        _ => panic!("ApiResource only supports structs"),
    };

    let (main_fields, get_fields, post_fields, patch_fields) = process_fields(fields);

    let get_name = format_ident!("{}Get", name);
    let post_name = format_ident!("{}Post", name);
    let patch_name = format_ident!("{}Patch", name);
    let endpoint = format_ident!("{}", plural_name);

    let expanded = quote! {
        #[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, tabled::Tabled)]
        pub struct #name {
            #main_fields
        }

        #[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
        pub struct #get_name {
            #get_fields
        }

        #[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
        pub struct #post_name {
            #post_fields
        }

        #[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
        pub struct #patch_name {
            #patch_fields
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.id)
            }
        }

        impl crate::resources::ApiResource for #name {
            type GetParams = #get_name;
            type GetOutput = #name;
            type PostParams = #post_name;
            type PostOutput = #name;
            type PatchParams = #patch_name;
            type PatchOutput = #name;
            type DeleteParams = ();
            type DeleteOutput = ();

            fn endpoint(&self) -> crate::endpoints::Endpoint {
                crate::endpoints::Endpoint::#endpoint
            }

            fn build_params(filters: Vec<(String, crate::types::FilterOperator, String)>) -> Vec<crate::types::QueryFilter> {
                let mut queries = vec![];
                for (field, op, value) in filters {
                    let key = format!("{}__{}", field, op);
                    queries.push(crate::types::QueryFilter {
                        key,
                        value,
                    });
                }
                queries
            }
        }
    };

    TokenStream::from(expanded)
}

fn process_fields(
    fields: &Punctuated<syn::Field, Comma>,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let mut main_fields = proc_macro2::TokenStream::new();
    let mut get_fields = proc_macro2::TokenStream::new();
    let mut post_fields = proc_macro2::TokenStream::new();
    let mut patch_fields = proc_macro2::TokenStream::new();

    for field in fields {
        let name = &field.ident;
        let ty = &field.ty;
        let is_read_only = field.attrs.iter().any(|attr| {
            if attr.path().is_ident("api") {
                if let Meta::List(list) = &attr.meta {
                    if let Ok(nested) =
                        list.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                    {
                        return nested.iter().any(
                            |meta| matches!(meta, Meta::Path(path) if path.is_ident("read_only")),
                        );
                    }
                }
            }
            false
        });

        let is_post_only = field.attrs.iter().any(|attr| {
            if attr.path().is_ident("api") {
                if let Meta::List(list) = &attr.meta {
                    if let Ok(nested) =
                        list.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                    {
                        return nested.iter().any(
                            |meta| matches!(meta, Meta::Path(path) if path.is_ident("post_only")),
                        );
                    }
                }
            }
            false
        });

        let is_optional = field.attrs.iter().any(|attr| {
            if attr.path().is_ident("api") {
                if let Meta::List(list) = &attr.meta {
                    if let Ok(nested) =
                        list.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                    {
                        return nested.iter().any(
                            |meta| matches!(meta, Meta::Path(path) if path.is_ident("optional")),
                        );
                    }
                }
            }
            false
        });

        let is_as_id = field.attrs.iter().any(|attr| {
            if attr.path().is_ident("api") {
                if let Meta::List(list) = &attr.meta {
                    if let Ok(nested) =
                        list.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                    {
                        return nested.iter().any(
                            |meta| matches!(meta, Meta::Path(path) if path.is_ident("as_id")),
                        );
                    }
                }
            }
            false
        });

        let id_field_name = if is_as_id {
            format!("{}_id", name.as_ref().unwrap().to_string())
        } else {
            name.as_ref().unwrap().to_string()
        };
        let id_field_ident = syn::Ident::new(&id_field_name, proc_macro2::Span::call_site());

        if !is_post_only {
            if is_optional {
                main_fields.extend(quote! {
                    #[tabled(display_with = "crate::resources::display_option")]
                    pub #name: Option<#ty>,
                });
            } else {
                main_fields.extend(quote! { pub #name: #ty, });
            }
            get_fields.extend(quote! { pub #id_field_ident: Option<#ty>, });
        }

        if is_post_only {
            post_fields.extend(quote! { pub #id_field_ident: #ty, });
        } else if !is_read_only {
            if is_as_id {
                let id_type = if is_optional {
                    quote!(Option<i32>)
                } else {
                    quote!(i32)
                };
                patch_fields.extend(quote! { pub #id_field_ident: #id_type, });
                post_fields.extend(quote! { pub #id_field_ident: #id_type, });
            } else {
                patch_fields.extend(quote! { pub #id_field_ident: Option<#ty>, });
                if !is_optional {
                    post_fields.extend(quote! { pub #id_field_ident: #ty, });
                } else {
                    post_fields.extend(quote! { pub #id_field_ident: Option<#ty>, });
                }
            }
        }
    }

    (main_fields, get_fields, post_fields, patch_fields)
}
