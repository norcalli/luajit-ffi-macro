extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens as _};
use syn::{parse_macro_input, FnArg, Item, ItemMod, ReturnType, Signature};

#[proc_macro_attribute]
pub fn luajit_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_mod = parse_macro_input!(item as ItemMod); // Parse the module
    let module_name = &input_mod.ident;
    let mut ffi_decls = Vec::new();
    let mut new_items = Vec::new();

    // Iterate over the items in the module
    for item in input_mod.content.unwrap().1 {
        if let Item::Fn(item_fn) = &item {
            if is_extern_c(&item_fn.sig) {
                let fn_name = &item_fn.sig.ident;
                let inputs = &item_fn.sig.inputs;
                let output = &item_fn.sig.output;
                let lua_ffi_decl = generate_ffi_declaration(fn_name, inputs, output);
                ffi_decls.push(lua_ffi_decl);
            }
        }
        new_items.push(quote! { #item });
    }

    let ffi_decls_str = ffi_decls.join("\n") + "\0";
    let ffi_function_name = format_ident!("{}_luajit_ffi_decls", module_name);

    let expanded = quote! {
        mod #module_name {
            use super::*;

            #(#new_items)*

            #[no_mangle]
            pub extern "C" fn #ffi_function_name() -> *const std::os::raw::c_char {
                static DECLS: &'static str = #ffi_decls_str;
                DECLS.as_ptr() as *const std::os::raw::c_char
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_extern_c(sig: &Signature) -> bool {
    sig.abi.as_ref().map_or(false, |abi| {
        abi.name.as_ref().map_or(false, |name| name.value() == "C")
    })
}

/* fn generate_ffi_declaration(
    fn_name: &syn::Ident,
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    output: &ReturnType,
) -> String {
    let args = inputs
        .iter()
        .map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                format!(
                    "{} {}",
                    quote! {#pat_type.ty},
                    pat_type.pat.to_token_stream()
                )
            } else {
                String::new()
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    let output_type = match output {
        ReturnType::Default => "void".to_string(),
        ReturnType::Type(_, ty) => quote! {#ty}.to_string(),
    };

    format!("{} {}({});", output_type, fn_name, args)
} */

fn generate_ffi_declaration(
    fn_name: &syn::Ident,
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    output: &ReturnType,
) -> String {
    let args = inputs
        .iter()
        .map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                format!(
                    "{} {}",
                    rust_to_c_type(&pat_type.ty),
                    pat_type.pat.to_token_stream()
                )
            } else {
                String::new()
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    let output_type = match output {
        ReturnType::Default => "void".to_string(),
        ReturnType::Type(_, ty) => rust_to_c_type(&ty),
    };

    format!("{} {}({});", output_type, fn_name, args)
}

fn rust_to_c_type(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let path = type_path.path.segments.last().unwrap().ident.to_string();
            match path.as_str() {
                "i32" => "int32_t",
                "u32" => "uint32_t",
                "i64" => "int64_t",
                "u64" => "uint64_t",
                "f64" => "double",
                "f32" => "float",
                _ => &path,
            }
            .to_string()
        }
        _ => "void".to_string(), // default to void for unsupported types
    }
}
