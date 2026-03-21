use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse2, parse::Parse, parse::ParseStream, Ident, ItemStruct, LitInt, Result, Token,
};

struct GameValueArgs {
    min: i64,
    max: i64,
}

// Parse `min = <lit>, max = <lit>` (order-independent, comma-separated)
impl Parse for GameValueArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut min: Option<i64> = None;
        let mut max: Option<i64> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _eq: Token![=] = input.parse()?;
            let val: LitInt = input.parse()?;
            let n: i64 = val.base10_parse()?;

            match key.to_string().as_str() {
                "min" => min = Some(n),
                "max" => max = Some(n),
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown key `{other}`, expected `min` or `max`"),
                    ))
                }
            }

            if !input.is_empty() {
                let _comma: Token![,] = input.parse()?;
            }
        }

        Ok(GameValueArgs {
            min: min.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing `min` argument")
            })?,
            max: max.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "missing `max` argument")
            })?,
        })
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let args: GameValueArgs = parse2(attr)?;
    let input: ItemStruct = parse2(item)?;

    let name = &input.ident;
    let vis = &input.vis;

    // Extract the inner type from the newtype (single unnamed field)
    let inner_ty = match &input.fields {
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            &fields.unnamed.first().unwrap().ty
        }
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "#[game_value] requires a newtype struct with exactly one unnamed field",
            ))
        }
    };

    let min = args.min;
    let max = args.max;
    let min_lit = proc_macro2::Literal::i64_unsuffixed(min);
    let max_lit = proc_macro2::Literal::i64_unsuffixed(max);
    let name_str = name.to_string();

    let expanded = quote! {
        #[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
        #vis struct #name(#inner_ty);

        impl #name {
            /// The minimum valid value.
            pub const MIN: #inner_ty = #min_lit as #inner_ty;
            /// The maximum valid value.
            pub const MAX: #inner_ty = #max_lit as #inner_ty;

            /// Create a new value, clamping at bounds. Always succeeds.
            /// This is the primary constructor — safe, zero-ceremony.
            #[inline]
            pub fn new(val: #inner_ty) -> Self {
                let clamped = if (val as i64) < #min_lit {
                    #min_lit as #inner_ty
                } else if (val as i64) > #max_lit {
                    #max_lit as #inner_ty
                } else {
                    val
                };
                Self(clamped)
            }

            /// Validate a value, returning Err if out of bounds.
            /// Use at trust boundaries (deserialization, user input).
            pub fn validate(val: #inner_ty) -> ::std::result::Result<Self, ::std::string::String> {
                let n = val as i64;
                if n < #min_lit || n > #max_lit {
                    Err(::std::format!(
                        "{}: value {} is out of bounds [{}, {}]",
                        #name_str, val, #min_lit, #max_lit
                    ))
                } else {
                    Ok(Self(val))
                }
            }

            #[inline]
            pub fn get(self) -> #inner_ty {
                self.0
            }
        }

        impl ::std::ops::Deref for #name {
            type Target = #inner_ty;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serde::Serialize::serialize(&self.0, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let val = <#inner_ty as serde::Deserialize<'de>>::deserialize(deserializer)?;
                Self::validate(val).map_err(serde::de::Error::custom)
            }
        }
    };

    Ok(expanded)
}
