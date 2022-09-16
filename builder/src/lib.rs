use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, AngleBracketedGenericArguments,
    DataStruct, DeriveInput, Fields, GenericArgument, Ident, Path, PathArguments, PathSegment,
    Type, TypePath,
};

fn is_option(field_type: &Type) -> bool {
    match field_type {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => match segments.first() {
            Some(path_seg) => path_seg.ident == Ident::new("Option", path_seg.ident.span()),
            None => false,
        },
        _ => false,
    }
}

fn option_inner(field_type: &Type) -> Punctuated<GenericArgument, Comma> {
    match field_type {
        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => match segments.first() {
            Some(PathSegment {
                arguments:
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }),
                ..
            }) => args.clone(),
            _ => panic!(""),
        },
        _ => panic!(""),
    }
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident: source_ident,
        data,
        ..
    } = parse_macro_input!(input as DeriveInput);
    // eprintln!("DeriveInput: {:#?}", input_tree);

    let builder_ident = format_ident!("{}Builder", source_ident);

    let named_fields = match data {
        syn::Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(field_named) => field_named.named,
            _ => panic!("No"),
        },
        _ => panic!("unsupported data type"),
    };

    // println!("Tokens (named_fields): {:#?}", named_fields);

    let struct_names_types = named_fields
        .iter()
        .map(|f| {
            let ident = f.ident.clone().unwrap();
            let field_type = f.ty.clone();
            (ident, field_type)
        })
        .collect::<Vec<_>>();

    let new_struct_fields = struct_names_types
        .iter()
        .map(|(ident, field_type)| {
            if is_option(field_type) {
                quote! {
                    #ident: #field_type
                }
            } else {
                quote! {
                    #ident: Option<#field_type>
                }
            }
        })
        .collect::<Vec<_>>();

    let new_builder_init = struct_names_types
        .iter()
        .map(|(ident, _field_type)| {
            quote! {
                #ident: None
            }
        })
        .collect::<Vec<_>>();

    let build_inner = struct_names_types
        .iter()
        .map(|(ident, field_type)| {
            if is_option(field_type) {
                quote! {
                    #ident: self.#ident.clone()
                }
            } else {
                quote! {
                    #ident: self.#ident.clone().ok_or_else(|| std::boxed::Box::new(std::fmt::Error))?
                }
            }
        })
        .collect::<Vec<_>>();

    let new_setter_functions = struct_names_types
        .iter()
        .map(|(ident, field_type)| {
            let reduced_field = if is_option(field_type) {
                // println!("field_type: {:#?}", field_type);
                let inner_type = option_inner(field_type);
                quote! { #inner_type }
            } else {
                quote! { #field_type }
            };
            quote! {
                fn #ident(&mut self, #ident: #reduced_field) -> &mut Self {
                    self.#ident = Some(#ident);
                    self
                }
            }
        })
        .collect::<Vec<_>>();

    // let Data::Struct(fields) = input_tree.data;

    let tokens = quote! {
        use std::error::Error;

        pub struct #builder_ident {
            #(#new_struct_fields),*
        }

        impl #builder_ident {
            fn build(&mut self) -> Result<#source_ident, Box<dyn Error>> {
                Ok(#source_ident {
                    #(#build_inner),*
                })
            }

            #(#new_setter_functions)*
        }

        impl #source_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#new_builder_init), *
                }
            }
        }
    };

    tokens.into()
}
