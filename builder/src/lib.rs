use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, PathArguments, Type, TypePath};


fn is_option_type(ty: &Type) -> bool {
    // First, check if it's a Type::Path
    if let Type::Path(TypePath { qself: None, path }) = ty {
        // Then check if the path has at least one segment
        if let Some(segment) = path.segments.first() {
            // Check if the first segment is "Option"
            if segment.ident == "Option" {
                // Finally, check if it has angle-bracketed arguments
                if let PathArguments::AngleBracketed(_) = &segment.arguments {
                    return true;
                }
            }
        }
    }
    false
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

    let setter_methods = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;


        if is_option_type(field_type) {
            quote! {
                pub fn #field_name(&mut self, #field_name: Option<#field_type>) -> &mut Self {
                    self.#field_name = Some(#field_name);
                    self
                }
            }
        } else {
            quote! {
                pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                    self.#field_name = Some(#field_name);
                    self
                }
            }
        }

    });

    let field_defs = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        quote! {
            #field_name: Option<#field_type>,
        }
    });

    let initialize_fields = fields.iter().map(|field| {
        let field_name = &field.ident;

        quote! {
            #field_name: None,
        }
    });


    let build_method = {

        let field_checks = fields.iter().map(|field| {
            let field_name = &field.ident;
            let field_type = &field.ty;

            if is_option_type(field_type) {
                quote! {
                    let #field_name = self.#field_name.clone().flatten().take();
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
                    // Initialize fields to None
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
