use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::{quote, format_ident};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input_tree = parse_macro_input!(input as DeriveInput);
    // eprintln!("DeriveInput: {:#?}", input_tree);

    let source_ident = input_tree.ident;
    let builder_ident = format_ident!("{}Builder", source_ident);

    let tokens = quote! {

        pub struct #builder_ident {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }

        impl #source_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    executable: None,
                    args: None,
                    env: None, 
                    current_dir: None
                }
            }
        }
    };

    tokens.into()
}
