//! A macro to replace function parameters types and convert them to the original type using
//! TryFrom.
//!
//! The original motivation for this crate was to use it with wasm-bindgen, so a function can .
//!
//! # Example
//!
//! ```
//! #[macro_use]
//! extern crate convert_params;
//!
//! struct Orig {}
//!
//! #[derive(Debug)]
//! struct Foo {}
//!
//! impl TryFrom<Orig> for Foo {
//!     type Error = &'static str;
//!     fn try_from(_v: Orig) -> Result<Self, Self::Error> {
//!         Ok(Foo {})
//!     }
//! }
//!
//! #[derive(Debug, PartialEq)]
//! struct Error {
//!     s: &'static str,
//! }
//!
//! impl From<&'static str> for Error {
//!     fn from(s: &'static str) -> Self {
//!         Error { s }
//!     }
//! }
//!
//! #[convert_args(_value1: Orig, _value2: Orig)]
//! fn example(i: u32, _value1: Foo, _value2: Foo) -> Result<(), Error> {
//!     Ok(())
//! }
//!
//! fn main() {
//!     assert!(example(42, Orig {}, Orig {}).is_ok());
//! }
//! ```
//!
//! `example` to:
//!
//! ```ignore
//! fn example(i: u32, _value1: Orig, _value2: Orig) -> Result<(), Error> {
//!     let _value2 = {
//!         let _value2: Orig = _value2.into();
//!         <Foo as std::convert::TryFrom<_>>::try_from(_value2)?
//!     };
//!     let _value1 = {
//!         let _value1: Orig = _value1.into();
//!         <Foo as std::convert::TryFrom<_>>::try_from(_value1)?
//!     };
//!     Ok(())
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input, punctuated::Punctuated, FnArg, Ident,
    ItemFn, Pat, Result as SynResult, Token, Type,
};

struct ConvertArgsItem {
    arg_name: Ident,
    new_type: Type,
}

struct ConvertArgs {
    args: Punctuated<ConvertArgsItem, Token![,]>,
}

impl Parse for ConvertArgsItem {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let arg_name = input.parse()?;
        input.parse::<Token![:]>()?;
        let new_type = input.parse()?;
        Ok(ConvertArgsItem { arg_name, new_type })
    }
}

impl Parse for ConvertArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let args = Punctuated::<ConvertArgsItem, Token![,]>::parse_terminated(input)?;
        Ok(ConvertArgs {
            args: args.into_iter().collect(),
        })
    }
}

#[proc_macro_attribute]
pub fn convert_args(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ConvertArgs { args } = parse_macro_input!(attr);

    let mut func = parse_macro_input!(item as ItemFn);
    let inputs = &mut func.sig.inputs;
    let block = &mut func.block;

    for ConvertArgsItem { arg_name, new_type } in args.into_iter().rev() {
        if let Some(FnArg::Typed(pat_type)) = inputs.iter_mut().find(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    return pat_ident.ident == arg_name;
                }
            }
            false
        }) {
            let original_type = pat_type.ty.clone();
            pat_type.ty = new_type.clone().into();

            let stmt: syn::Stmt = syn::parse2(quote! {
                let #arg_name = {
                    let #arg_name: #new_type = #arg_name.into();
                    <#original_type as std::convert::TryFrom<_>>::try_from(#arg_name)?
                };
            })
            .unwrap();

            block.stmts.insert(0, stmt);
        } else {
            panic!("No argument named {}", arg_name);
        }
    }

    TokenStream::from(quote!(#func))
}
