use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error};

#[proc_macro_derive(Packet)]
pub fn packet_derive(input: TokenStream) -> TokenStream {
    let encoder_stream = proc_macro2::TokenStream::from(encoder_derive(input.clone()));
    let decoder_stream = proc_macro2::TokenStream::from(decoder_derive(input.clone()));

    // Parse the input token stream as a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Generate code based on the packet id and flow
    let expanded = quote! {
        #encoder_stream
        #decoder_stream

        impl blocky_net::packet::Packet for #name {}
    };

    // Convert the quote into a TokenStream
    TokenStream::from(expanded)
}

#[proc_macro_derive(Encoder)]
pub fn encoder_derive(input: TokenStream) -> TokenStream {
    // Parse the input token stream as a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data_struct) => &data_struct.fields,
        _ => {
            return Error::new_spanned(&input, "Encoder derive only supports structs")
                .to_compile_error()
                .into()
        }
    };

    let field_names = fields.iter().map(|f| &f.ident).collect::<Vec<_>>();

    // Generate code based on the packet id and flow
    let expanded = quote! {
        impl blocky_net::encoder::Encoder for #name {
            fn byte_len(&self) -> usize {
                #(self.#field_names.byte_len() +)*
                0
            }

            fn encode<T: std::io::Write>(&self, buf: &mut T) -> anyhow::Result<()> {
                #(self.#field_names.encode(buf)?;)*
                Ok(())
            }
        }
    };

    // Convert the quote into a TokenStream
    TokenStream::from(expanded)
}

#[proc_macro_derive(Decoder)]
pub fn decoder_derive(input: TokenStream) -> TokenStream {
    // Parse the input token stream as a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data_struct) => &data_struct.fields,
        _ => {
            return Error::new_spanned(&input, "Decoder derive only supports structs")
                .to_compile_error()
                .into()
        }
    };

    let field_names = fields.iter().map(|f| &f.ident).collect::<Vec<_>>();

    // Generate code based on the packet id and flow
    let expanded = quote! {
        impl blocky_net::decoder::Decoder for #name {
            fn decode<T: std::io::Read>(buf: &mut T) -> anyhow::Result<Self> {
                Ok(Self {
                    #(#field_names: blocky_net::decoder::Decoder::decode(buf)?,)*
                })
            }
        }
    };

    // Convert the quote into a TokenStream
    TokenStream::from(expanded)
}
