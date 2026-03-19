use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    bracketed,
    parse2,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, ItemStruct, Result, Token,
};

struct GameEntityAttr {
    required_fields: Vec<Ident>,
}

impl Parse for GameEntityAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        if ident != "requires" {
            return Err(syn::Error::new(ident.span(), "expected `requires`"));
        }
        let _eq: Token![=] = input.parse()?;
        let content;
        bracketed!(content in input);
        let fields: Punctuated<Ident, Token![,]> =
            content.parse_terminated(Ident::parse, Token![,])?;
        Ok(GameEntityAttr {
            required_fields: fields.into_iter().collect(),
        })
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let attr: GameEntityAttr = parse2(attr)?;
    let input: ItemStruct = parse2(item)?;
    let name = &input.ident;
    let vis = &input.vis;
    let builder_name = format_ident!("{}Builder", name);

    let fields = match &input.fields {
        syn::Fields::Named(f) => &f.named,
        _ => {
            return Err(syn::Error::new(
                name.span(),
                "game_entity requires named fields",
            ))
        }
    };

    let required_set: std::collections::HashSet<String> =
        attr.required_fields.iter().map(|i| i.to_string()).collect();

    let mut req_fields = Vec::new();
    let mut opt_fields = Vec::new();
    for f in fields {
        let fname = f.ident.as_ref().unwrap();
        let fty = &f.ty;
        if required_set.contains(&fname.to_string()) {
            req_fields.push((fname.clone(), fty.clone()));
        } else {
            opt_fields.push((fname.clone(), fty.clone()));
        }
    }

    let n = req_fields.len();

    // Const generic params: const HAS_0: bool, const HAS_1: bool, ...
    let const_params: Vec<_> = (0..n).map(|i| format_ident!("HAS_{}", i)).collect();

    let const_param_decls: Vec<_> = const_params
        .iter()
        .map(|p| quote! { const #p: bool })
        .collect();

    // Builder fields: required and optional as Option<T>
    let req_builder_fields: Vec<_> = req_fields
        .iter()
        .map(|(fname, fty)| quote! { #fname: Option<#fty> })
        .collect();

    let opt_builder_fields: Vec<_> = opt_fields
        .iter()
        .map(|(fname, fty)| quote! { #fname: Option<#fty> })
        .collect();

    // new() — all false, all None
    let false_params: Vec<_> = (0..n).map(|_| quote! { false }).collect();
    let none_inits: Vec<_> = req_fields
        .iter()
        .chain(opt_fields.iter())
        .map(|(fname, _)| quote! { #fname: None })
        .collect();

    // Per-required-field setter methods
    let setter_methods: Vec<_> = req_fields
        .iter()
        .enumerate()
        .map(|(i, (fname, fty))| {
            let input_params: Vec<_> = const_params.iter().map(|p| quote! { #p }).collect();
            let output_params: Vec<_> = const_params
                .iter()
                .enumerate()
                .map(|(j, p)| {
                    if j == i {
                        quote! { true }
                    } else {
                        quote! { #p }
                    }
                })
                .collect();
            let copy_fields: Vec<_> = req_fields
                .iter()
                .chain(opt_fields.iter())
                .map(|(f, _)| {
                    if f == fname {
                        quote! { #f: Some(val) }
                    } else {
                        quote! { #f: self.#f }
                    }
                })
                .collect();

            quote! {
                impl<#(#const_param_decls),*> #builder_name<#(#input_params),*> {
                    pub fn #fname(self, val: #fty) -> #builder_name<#(#output_params),*> {
                        #builder_name {
                            #(#copy_fields),*
                        }
                    }
                }
            }
        })
        .collect();

    // Optional field setters
    let opt_setter_methods: Vec<_> = opt_fields
        .iter()
        .map(|(fname, fty)| {
            let all_params: Vec<_> = const_params.iter().map(|p| quote! { #p }).collect();
            let copy_fields: Vec<_> = req_fields
                .iter()
                .chain(opt_fields.iter())
                .map(|(f, _)| {
                    if f == fname {
                        quote! { #f: Some(val) }
                    } else {
                        quote! { #f: self.#f }
                    }
                })
                .collect();
            quote! {
                impl<#(#const_param_decls),*> #builder_name<#(#all_params),*> {
                    pub fn #fname(self, val: #fty) -> #builder_name<#(#all_params),*> {
                        #builder_name {
                            #(#copy_fields),*
                        }
                    }
                }
            }
        })
        .collect();

    // build() — only when all const generics are true
    let true_params: Vec<_> = (0..n).map(|_| quote! { true }).collect();
    let build_req_fields: Vec<_> = req_fields
        .iter()
        .map(|(fname, _)| quote! { #fname: self.#fname.unwrap() })
        .collect();
    let build_opt_fields: Vec<_> = opt_fields
        .iter()
        .map(|(fname, _)| quote! { #fname: self.#fname.unwrap_or_default() })
        .collect();

    let all_orig_fields: Vec<_> = fields
        .iter()
        .map(|f| {
            let fname = &f.ident;
            let fty = &f.ty;
            quote! { #fname: #fty }
        })
        .collect();

    let output = quote! {
        #[derive(Debug, Clone)]
        #vis struct #name {
            #(pub #all_orig_fields),*
        }

        #vis struct #builder_name<#(#const_param_decls),*> {
            #(#req_builder_fields,)*
            #(#opt_builder_fields,)*
        }

        impl #name {
            pub fn builder() -> #builder_name<#(#false_params),*> {
                #builder_name {
                    #(#none_inits),*
                }
            }
        }

        #(#setter_methods)*

        #(#opt_setter_methods)*

        impl #builder_name<#(#true_params),*> {
            pub fn build(self) -> #name {
                #name {
                    #(#build_req_fields,)*
                    #(#build_opt_fields,)*
                }
            }
        }
    };

    Ok(output)
}
