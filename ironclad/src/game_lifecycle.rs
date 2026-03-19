use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::ParseStream, parse2, Ident, ItemStruct, Result,
    Token,
};

struct States {
    names: Vec<Ident>,
}

impl syn::parse::Parse for States {
    fn parse(input: ParseStream) -> Result<Self> {
        // Accept either `->` (token) or bare idents separated by `->`.
        // syn's Punctuated with custom separator isn't available for `->`,
        // so we parse manually.
        let mut names = vec![input.parse::<Ident>()?];
        while input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            names.push(input.parse::<Ident>()?);
        }
        Ok(States { names })
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let States { names: states } = parse2::<States>(attr)?;
    let base: ItemStruct = parse2(item)?;
    let struct_name = &base.ident;
    let vis = &base.vis;

    if states.is_empty() {
        return Err(syn::Error::new_spanned(
            struct_name,
            "game_lifecycle requires at least one state",
        ));
    }

    // 1. Zero-sized marker structs for every state.
    let marker_structs = states.iter().map(|s| {
        quote! {
            #[derive(Debug, Clone, Copy)]
            #vis struct #s;
        }
    });

    // 2. Generic wrapper struct.
    let wrapper = quote! {
        #[derive(Debug, Clone)]
        #vis struct #struct_name<State> {
            _state: ::std::marker::PhantomData<State>,
        }
    };

    // 3. new() constructor returning first state.
    let first_state = &states[0];
    let constructor = quote! {
        impl #struct_name<#first_state> {
            #vis fn new() -> Self {
                #struct_name { _state: ::std::marker::PhantomData }
            }
        }
    };

    // 4. Transition impls: each state except the last gets advance() -> next.
    let transitions = states.windows(2).map(|pair| {
        let current = &pair[0];
        let next = &pair[1];
        quote! {
            impl #struct_name<#current> {
                #vis fn advance(self) -> #struct_name<#next> {
                    #struct_name { _state: ::std::marker::PhantomData }
                }
            }
        }
    });

    let output = quote! {
        #(#marker_structs)*
        #wrapper
        #constructor
        #(#transitions)*
    };

    Ok(output)
}
