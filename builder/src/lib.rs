use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput};


fn extract_type_from_option(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        // Make sure there's no qualified self path (like <T as Trait>::Option)
        if type_path.qself.is_none() {
            let path = &type_path.path;
            if let Some(segment) = path.segments.first() {
                if segment.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            return Some(inner_type);
                        }
                    }
                }
            }
        }
    }
    None
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let builder_name = format_ident!("{}Builder", name);

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => panic!("Only structs are supported"),
    };

    let field_defs = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        if let Some(inner_type) = extract_type_from_option(field_type) {
            quote! {
                #field_name: Option<#inner_type>,
            }
        } else {
            quote! {
                #field_name: Option<#field_type>,
            }
        }

    });

    let initialize_fields = fields.iter().map(|field| {
        let field_name = &field.ident;

        quote! {
            #field_name: None,
        }
    });


    let setter_methods = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        if let Some(inner_type) = extract_type_from_option(field_type) {
            quote! {
                pub fn #field_name(&mut self, #field_name: #inner_type) -> &mut Self {
                    self.#field_name = Some(#field_name);
                    self
                }
            }
        } else  {
            quote! {
                pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                    self.#field_name = Some(#field_name);
                    self
                }
            }
        }

    });

    let build_method = {

        let field_checks = fields.iter().map(|field| {
            let field_name = &field.ident;
            let field_type = &field.ty;

            if let Some(_) = extract_type_from_option(field_type) {
                quote! {
                    let #field_name = self.#field_name.clone();
                }
            } else {
                quote! {
                    let #field_name = self.#field_name.take().ok_or_else(||
                        format!("{} is not set", stringify!(#field_name)))?;
                }
            }
        });

        let field_inits = fields.iter().map(|field| {
            let field_name = &field.ident;

            quote! {
                #field_name,
            }
        });

        quote! {
            pub fn build(&mut self) -> Result<#name, String> {
                #(#field_checks)*

                Ok(#name {
                    #(#field_inits)*
                })
            }
        }

    };

    let expanded = quote! {
        pub struct #builder_name {
            #(#field_defs)*
        }

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#initialize_fields)*
                }
            }
        }

        impl #builder_name {
            #(#setter_methods)*


            #build_method
        }
    };


    expanded.into()
}
