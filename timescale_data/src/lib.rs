use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, AttributeArgs, Fields, Index, ItemStruct};

#[proc_macro_attribute]
pub fn timescale_data(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let args = parse_macro_input!(args as AttributeArgs);

    if let Some(arg) = args.first() {
        let len = Index::from(args.len());
        return TokenStream::from(quote_spanned! {arg.span()=>
            compile_error!(concat!("this function takes 0 arguments but ", stringify!(#len), " arguments were supplied"));
        });
    }

    TokenStream::from(match input.fields {
        Fields::Named(fields) => {
            let (name, attrs, vis, generics) =
                (input.ident, input.attrs, input.vis, input.generics);
            let fields = fields.named.into_iter();

            let serializers = fields.clone().map(|f| {
                let name = f.ident;

                quote! {
                    s.serialize_field(concat!(stringify!(#name), "_x"), &self.#name[0])?;
                    s.serialize_field(concat!(stringify!(#name), "_y"), &self.#name[1])?;
                    s.serialize_field(concat!(stringify!(#name), "_z"), &self.#name[2])?;
                }
            });

            let fields_count = serializers.len() + 1;

            quote! {
                #(#attrs)*
                #vis struct #name #generics {
                    /// The time since the start of the simulation that this data point was logged
                    pub time: f64,
                    #(#fields),*
                }

                #[doc(hidden)]
                #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
                const _: () = {
                    extern crate serde as _serde;
                    use _serde::ser::SerializeStruct;

                    impl _serde::Serialize for #name {
                        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where
                            S: _serde::Serializer,
                        {
                            let mut s = serializer.serialize_struct(stringify!(#name), #fields_count)?;
                            s.serialize_field("time", &self.time)?;
                            #(#serializers)*
                            s.end()
                        }
                    }
                };
            }
        }
        _ => quote_spanned! {input.span()=>
            compile_error!("Structs must have named fields");
        },
    })
}
