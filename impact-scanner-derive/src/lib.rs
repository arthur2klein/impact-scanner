extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

mod helper;

#[proc_macro_derive(TestBuilder, attributes(builder))]
pub fn test_builder_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(fields),
        ..
    }) = &input.data
    {
        &fields.named
    } else {
        unimplemented!("Only named fields supported")
    };

    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { #name: Option<#ty> }
    });

    let create_defaults = fields.iter().map(|f| {
        let name = &f.ident;
        let default = helper::get_default_value(&f.attrs)
            .map(|val| quote! { Some(#val) })
            .unwrap_or_else(|| quote! { Some(Default::default()) });
        quote! { #name: #default }
    });

    let with_methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        let method = format_ident!(
            "with{}",
            helper::pascal_case(&name.as_ref().unwrap().to_string())
        );
        quote! {
            pub fn #method(mut self, #name: #ty) -> Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name: self.#name.unwrap() }
    });

    let expanded = quote! {
            pub struct #builder_name {
                #(#builder_fields,)*
            }
    impl #builder_name { pub fn create() -> Self { Self { #(#create_defaults,)* } }
                #(#with_methods)*

                pub fn build(self) -> #struct_name {
                    #struct_name {
                        #(#build_fields,)*
                    }
                }
            }
        };

    TokenStream::from(expanded)
}
