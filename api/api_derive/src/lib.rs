use proc_macro::{TokenStream};

use quote::{quote, ToTokens};
use syn::{Meta, Error, NestedMeta};
use syn::spanned::Spanned;

#[proc_macro_derive(Resource, attributes(resource))]
pub fn resources_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("failed to parse TokenStream when deriving trait api::Resource");
    match impl_resources_macro(&ast) {
        Ok(stream) => stream,
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_resources_macro(ast: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let meta: Meta = ast.attrs.iter().find_map(|attr| match attr.parse_meta() {
        Ok(m) => if m.path().is_ident("resource") {
            Some(Ok(m))
        } else {
            None
        },
        Err(err) => Some(Err(err)),
    }).map_or(Ok(None), |m| m.map(Some))?
        .ok_or_else(|| Error::new(name.span(), "missing attribute 'resource'"))?;

    let resource_attrs = match meta {
        Meta::List(meta) => meta,
        _ => return Err(Error::new(name.span(), "attribute 'resource' has incorrect type")),
    };

    let mut status: Option<syn::Type> = None;
    let mut api_version: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut kind_plural: Option<String> = None;

    for attr in &resource_attrs.nested {
        let pair = match attr {
            NestedMeta::Meta(Meta::NameValue(ref pair)) => pair,
            _ => return Err(Error::new(resource_attrs.span(), format!("unsupported attribute argument {:?}", attr.to_token_stream()))),
        };

        if pair.path.is_ident("status") {
            if let syn::Lit::Str(ref s) = pair.lit {
                if s.value() != "()" {
                    status = None;
                } else {
                    status = s.parse().map(Some)?;
                }
            } else {
                return Err(Error::new(pair.span(), "resource status must be a string literal"));
            }
        } else if pair.path.is_ident("api_version") {
            if let syn::Lit::Str(ref s) = pair.lit {
                api_version = Some(s.value());
            } else {
                return Err(Error::new(pair.span(), "resource status must be a string literal"));
            }
        } else if pair.path.is_ident("kind") {
            if let syn::Lit::Str(ref s) = pair.lit {
                kind = Some(s.value());
            } else {
                return Err(Error::new(pair.span(), "resource status must be a string literal"));
            }
        } else if pair.path.is_ident("kind_plural") {
            if let syn::Lit::Str(ref s) = pair.lit {
                kind_plural = Some(s.value());
            } else {
                return Err(Error::new(pair.span(), "resource status must be a string literal"));
            }
        } else {
            return Err(Error::new(pair.span(), format!("unsupported attribute key '{}'", pair.path.to_token_stream())));
        }
    }

    let status_methods = if status.is_some() {
        quote! {
            fn status(&self) -> Option<&Self::Status> {
                self.status.as_ref()
            }

            fn status_mut(&mut self) -> Option<&mut Self::Status> {
                if self.status.is_none() {
                    self.status = Some(Default::default());
                }

                self.status.as_mut()
            }

            fn set_status(&mut self, status: Option<Self::Status>) {
                self.status = status;
            }
        }
    } else {
        quote! {
            fn status(&self) -> Option<&Self::Status> {
                None
            }

            fn status_mut(&mut self) -> Option<&mut Self::Status> {
                None
            }

            fn set_status(&mut self, status: Option<Self::Status>) {}
        }
    };

    let status = status.unwrap_or_else(|| syn::LitStr::new("()", resource_attrs.span()).parse().unwrap());
    let api_version = api_version.ok_or_else(|| Error::new(resource_attrs.span(), "api_version is missing"))?;
    let parts: Vec<&str> = api_version.split('/').collect();
    let api_group = parts.first().cloned().map(str::to_string).ok_or_else(|| Error::new_spanned(&resource_attrs, format!("invalid api_version '{}', missing group", api_version)))?;
    let api_version = parts.last().cloned().map(str::to_string).ok_or_else(|| Error::new_spanned(&resource_attrs, format!("invalid api_version '{}', missing version", api_version)))?;
    let kind = kind.ok_or_else(|| Error::new(resource_attrs.span(), "kind is missing"))?;
    let kind_plural = kind_plural.ok_or_else(|| Error::new(resource_attrs.span(), "kind_plural is missing"))?;

    let gen = quote! {
        impl #impl_generics Resource for #name #ty_generics #where_clause {
            type Status = #status;

            fn type_meta() -> TypeMeta {
                TypeMeta {
                    api_version: ::api::GroupVersion {
                        group: #api_group.to_string(),
                        version: #api_version.to_string(),
                    },
                    kind: #kind.to_string(),
                    kind_plural: #kind_plural.to_string(),
                }
            }

            fn reset_type_meta(&mut self) {
                self.type_meta = Self::type_meta();
            }

            fn metadata(&self) -> &Metadata {
                &self.metadata
            }

            fn metadata_mut(&mut self) -> &mut Metadata {
                &mut self.metadata
            }

            fn set_metadata(&mut self, metadata: Metadata) {
                self.metadata = metadata;
            }

            #status_methods
        }
    };
    Ok(gen.into())
}
