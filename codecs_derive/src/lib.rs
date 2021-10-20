use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attrs;

mod symbol;

mod aper;

#[proc_macro_derive(AperCodec, attributes(asn))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let codec_params = attrs::parse_ty_meta_as_codec_params(&ast.attrs);
    if codec_params.is_err() {
        return codec_params.err().unwrap().to_compile_error().into();
    }

    let codec_params = codec_params.unwrap();
    if codec_params.attr.is_none() {
        return syn::Error::new_spanned(ast.clone(), "Missing attribute 'asn' for the struct.")
            .to_compile_error()
            .into();
    }

    if codec_params.ty.is_none() {
        return syn::Error::new_spanned(
            codec_params.attr.as_ref().clone(),
            "Missing parameter 'type' for the attribute.",
        )
        .to_compile_error()
        .into();
    }

    aper::generate_decode(&ast, &codec_params)
}
