//! `APER` Code generation for ASN.1 INTEGER Type

use quote::quote;
use proc_macro;
use syn;


use crate::{attrs::TyCodecParams, utils};

fn generate_variant_decode_tokens(
    ast: &syn::DeriveInput,
    //lb: i128,
    //ub: i128,
    //ext: Option<&syn::LitBool>,
    //codec_encode_fn: proc_macro2::TokenStream,
    //codec_decode_fn: proc_macro2::TokenStream,
    //choice_encode_path: proc_macro2::TokenStream,
) -> Result<(Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>), syn::Error> {

    let mut decode_tokens = vec![];
    let mut encode_tokens = vec![];
    
    if let syn::Data::Enum(ref data) = ast.data {
        let mut key = 0i128;
        for variant in &data.variants {
            let variant_ident = &variant.ident;

            let variant_decode_token = quote! {
                #key => Ok(Self::#variant_ident),
                //0 => Ok(Self::CASHIER),
                //1 => Ok(Self::MANAGER),
            };


/*
            let variant_encode_token = quote! {
                Self::#variant_ident => Ok((#key)),
            };

*/
            let variant_encode_token = quote! {
                Self::#variant_ident => Ok(()),
            };
		/*

	    let variant_encode_token = quote! {
		Self::#variant_ident => {
		    #choice_encode_path(data, #lb, #ub, #ext, #key, false)?;
		    Self.#codec_encode_fn(data)
		}
	    };
*/

            decode_tokens.push(variant_decode_token);
            encode_tokens.push(variant_encode_token);
            key = key + 1;
        }
    }

    Ok((decode_tokens, encode_tokens))
}

pub(super) fn generate_aper_codec_for_asn_enumerated(
    ast: &syn::DeriveInput,
    params: &TyCodecParams,
    aligned: bool,
) -> proc_macro::TokenStream {
    let name = &ast.ident;
    
    log::trace!("generate_aper_codec_for_asn_enumerated");
    //println!("generate_aper_codec_for_asn_enumerated");

    let (lb, ub, ext) = utils::get_bounds_extensible_from_params(params);

    let (codec_path, codec_encode_fn, codec_decode_fn, ty_encode_path, ty_decode_path) = if aligned
    {
        (
            quote!(asn1_codecs::aper::AperCodec),
            quote!(aper_encode),
            quote!(aper_decode),
            quote!(asn1_codecs::aper::encode::encode_enumerated),
            quote!(asn1_codecs::aper::decode::decode_enumerated),
        )
    } else {
        (
            quote!(asn1_codecs::uper::UperCodec),
            quote!(uper_encode),
            quote!(uper_decode),
            quote!(asn1_codecs::uper::encode::encode_enumerated),
            quote!(asn1_codecs::uper::decode::decode_enumerated),
        )
    };

    let tokens = if let syn::Data::Struct(ref d) = &ast.data {

        let ty = match d.fields {
            syn::Fields::Unnamed(ref f) => {
                if f.unnamed.len() == 1 {
                    let first = f.unnamed.first().unwrap();
                    Some(first.ty.clone())
                } else {
                    None
                }
            }
            _ => None,
        };

        if ty.is_none() {
            return syn::Error::new_spanned(ast, format!("Couldn't determine type for {}.", name))
                .to_compile_error()
                .into();
        }


        quote! {

            impl #codec_path for #name {
                type Output = Self;

                fn #codec_decode_fn(data: &mut asn1_codecs::PerCodecData) -> Result<Self::Output, asn1_codecs::PerCodecError> {
                    log::trace!(concat!("decode: ", stringify!(#name)));

                    let decoded = #ty_decode_path(data, #lb, #ub, #ext)?;

                    Ok(Self(decoded.0 as #ty))
                }

                fn #codec_encode_fn(&self, data: &mut asn1_codecs::PerCodecData) -> Result<(), asn1_codecs::PerCodecError> {
                    log::trace!(concat!("encode: ", stringify!(#name)));

                    #ty_encode_path(data, #lb, #ub, #ext, self.0 as i128, false)
                }
            }
        }


    } else if let syn::Data::Enum(ref _data_enum) = &ast.data { 
    //} else if let syn::Data::Enum() = &ast.data { 

        let lb = params.lb.as_ref().unwrap().value().parse::<i128>().unwrap();
        let ub = params.ub.as_ref().unwrap().value().parse::<i128>().unwrap();

        let ext = params.ext.as_ref();

        let variant_tokens = generate_variant_decode_tokens(
            ast,
            //lb,
            //ub,
            //ext,
            //codec_encode_fn.clone(),
            //codec_decode_fn.clone(),
            //ty_encode_path,
        );
     

        if variant_tokens.is_err() {
            return variant_tokens.err().unwrap().to_compile_error().into();
        }

        let (variant_decode_tokens, variant_encode_tokens) = variant_tokens.unwrap();


        //println!("Bounds: {} {} \n", lb, ub);
        //println!("variants: {:#?} \n", variant_encode_tokens);

        //println!("enum impl aper codec:\n{}\n",tokens.to_string());
        let ret = quote! {

            impl #codec_path for #name {
                type Output = Self;

                fn #codec_decode_fn(data: &mut asn1_codecs::PerCodecData) -> Result<Self::Output, asn1_codecs::PerCodecError> {
                    log::trace!(concat!("decode: ", stringify!(#name)));

                    let (idx, _) = #ty_decode_path(data, Some(#lb) , Some(#ub), #ext)?;
                    //let idx= #ty_decode_path(data, Some(#lb) , Some(#ub), #ext)?;
                    //let decoded = #ty_decode_path(data, #lb, #ub, #ext)?;
                    match idx {
                        #(#variant_decode_tokens)*
                        _ => Err(asn1_codecs::PerCodecError::new(format!("Index {} is not a valid Choice Index", idx).as_str()))
                    }
                    /*
                    if !extended {
                        log::trace!("is not extended");
                        match idx {
                            #(#variant_decode_tokens)*
                            _ => Err(asn1_codecs::PerCodecError::new(format!("Index {} is not a valid Choice Index", idx).as_str()))
                    }
                    } else {
                        log::trace!("is extended");
                        Err(asn1_codecs::PerCodecError::new(format!("ENUM Additions not supported yet: {}::{}", stringify!(#name), idx)))
                    }
                    */
                }

                fn #codec_encode_fn(&self, data: &mut asn1_codecs::PerCodecData) -> Result<(), asn1_codecs::PerCodecError> {
                    log::trace!(concat!("encode: ", stringify!(#name)));

                    match self {
                        #(#variant_encode_tokens)*
                    }
                }
            }
        };
        

        //println!("{}", ret.to_string());
        ret



        /*
        syn::Error::new_spanned(ast, format!("{} as enum currently not implemented.", name))
            .to_compile_error()
            .into()
        */

    } else {
        syn::Error::new_spanned(ast, format!("{} Should be a Unit Struct or Enum", name))
            .to_compile_error()
            .into()
    };

/*


impl asn1_codecs::aper::AperCodec for employee::WorkRole{
    type Output = Self;
    fn aper_decode(
        data: &mut asn1_codecs::PerCodecData,
    ) -> Result<Self::Output, asn1_codecs::PerCodecError> {
        log::trace!(concat!("decode: ", stringify!(EmployeeChoice)));
        let (idx, extended) =
            asn1_codecs::aper::decode::decode_choice_idx(data, 0i128, 1i128, false)?;
        if !extended {
            match idx {
                0 => Ok(Self::CASHIER),
                1 => Ok(Self::MANAGER),
                2 => Ok(Self::CHEF),
                3 => Ok(Self::WAITER),
                _ => Err(asn1_codecs::PerCodecError::new(
                    format!("Index {} is not a valid Choice Index", idx).as_str(),
                )),
            }
        } else {
            Err(asn1_codecs::PerCodecError::new(
                "CHOICE Additions not supported yet.",
            ))
        }
    }
}



 * */
    
    //println!("--tokens--\n{}",tokens);
    tokens.into()
}
