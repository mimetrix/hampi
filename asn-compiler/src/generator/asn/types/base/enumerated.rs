//! Mainly 'generator' code for `Asn1ResolvedEnumerated`

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::error::Error;

use crate::generator::Generator;
use crate::resolver::asn::structs::types::base::Asn1ResolvedEnumerated;

use std::env;

impl Asn1ResolvedEnumerated {
    pub(crate) fn generate(
        &self,
        name: &str,
        generator: &mut Generator,
    ) -> Result<TokenStream, Error> {
        log::debug!("generate");
        let struct_name = generator.to_type_ident(name);
        let inner_type = generator.to_inner_type(self.bits, self.signed);

        let named_values = self.generate_named_values(generator)?;

        let mut ty_attributes = quote! { type = "ENUMERATED" };
        if self.extensible {
            ty_attributes.extend(quote! { , extensible = true });
        } else {
            //added to support asn1 enumerated types as enums
            ty_attributes.extend(quote! { , extensible = false});
        }
        let ub = format!("{}", self.named_root_values.len() - 1);
        ty_attributes.extend(quote! { , lb = "0" });
        ty_attributes.extend(quote! { , ub =  #ub  });

        let vis = generator.get_visibility_tokens();
        let dir = generator.generate_derive_tokens();


        let key = "ENUMS";
        let val = match env::var(key){
            Ok(val) => val,
            Err(e) => format!("{:?}",e),
        };

        let struct_tokens = match val.as_str() {
            "ENUMS" => 
            quote! {
                #dir
                #[asn(#ty_attributes)]
                #vis enum #struct_name {
                    #named_values
                }
            },
        
            _ => 
            quote! {
                #dir
                #[asn(#ty_attributes)]
                #vis struct #struct_name(#vis #inner_type);
                impl #struct_name {
                #named_values
                }
            },
        };
        
        log::debug!("enum_tokens\n{}",struct_tokens.to_string());
        Ok(struct_tokens)
    }

    fn generate_named_values(&self, generator: &Generator) -> Result<TokenStream, Error> {
        let mut tokens = TokenStream::new();

        let key = "ENUMS";
        let val = match env::var(key){
            Ok(val) => val,
            Err(e) => format!("{:?}",e),
        };

        for (name, value) in &self.named_root_values {
            let const_name = generator.to_const_ident(name);
            let value_literal = generator.to_isize_unsuffixed(self.bits, self.signed, *value);
            let const_tokens = match val.as_str() {
                "ENUMS" => 
                    quote!{
                        #const_name = #value_literal,
                    },
                _ => {

                    let ty = generator.to_inner_type(self.bits, self.signed);
                    let vis = generator.get_visibility_tokens();
                    quote! {
                        #vis const #const_name: #ty =  #value_literal ;
                    }
                },
            };
            tokens.extend(const_tokens);
        }

        Ok(tokens)
    }

    pub(crate) fn generate_ident_and_aux_type(
        &self,
        generator: &mut Generator,
        input: Option<&String>,
    ) -> Result<Ident, Error> {
        let unique_name = if input.is_none() {
            generator.get_unique_name("ENUMERATED")
        } else {
            input.unwrap().to_string()
        };

        let item = self.generate(&unique_name, generator)?;
        generator.aux_items.push(item);

        Ok(generator.to_type_ident(&unique_name))
    }
}
