use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    AttributeArgs, FnArg, Ident, Item, ItemMod, Lit, Meta, NestedMeta, Pat, PatType, ReturnType,
    Type, TypePath, parse_macro_input,
};

#[proc_macro_attribute]
pub fn register_functions(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute arguments
    let args = parse_macro_input!(attr as AttributeArgs);

    let mut map_ident: Option<Ident> = None;

    for arg in args {
        if let NestedMeta::Meta(Meta::NameValue(nv)) = arg {
            let ident = nv.path.get_ident().unwrap().to_string();
            match ident.as_str() {
                "map" => {
                    if let Lit::Str(litstr) = nv.lit {
                        map_ident = Some(Ident::new(&litstr.value(), litstr.span()));
                    }
                },
                _ => {
                    return syn::Error::new_spanned(nv.path, "Unknown argument")
                        .to_compile_error()
                        .into();
                },
            }
        }
    }

    let map_ident = map_ident.expect("map argument missing");
    let registry_ident = format_ident!("{}Registry", map_ident);

    // Parse module
    let mut module = parse_macro_input!(item as ItemMod);
    let mod_ident = &module.ident;

    let mut registrations = Vec::new();
    let mut wrapper_fns = Vec::new();

    if let Some((_, ref mut items)) = module.content {
        for item in items.iter_mut() {
            if let Item::Fn(f) = item {
                let fn_name = &f.sig.ident;
                let fn_name_str = fn_name.to_string();
                // Make wrapper name unique by including map name
                let wrapper_name = format_ident!(
                    "{}_{}_wrapper",
                    map_ident.to_string().to_ascii_lowercase(),
                    fn_name
                );

                // Collect argument info for casting
                let mut arg_casts = Vec::new();
                let mut arg_names = Vec::new();

                for (idx, arg) in f.sig.inputs.iter().enumerate() {
                    if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
                        let arg_name = if let Pat::Ident(pat_ident) = &**pat {
                            pat_ident.ident.clone()
                        } else {
                            format_ident!("arg{}", idx)
                        };

                        let arg_type = get_type(ty);
                        let cast_expr = match arg_type.as_str() {
                            "bool" => quote! {
                                let #arg_name: bool = args.get(#idx)
                                    .ok_or_else(|| format!("Missing argument {} for function {}", #idx, #fn_name_str))?
                                    .to_bool()?;
                            },
                            "i64" => quote! {
                                let #arg_name: i64 = args.get(#idx)
                                    .ok_or_else(|| format!("Missing argument {} for function {}", #idx, #fn_name_str))?
                                    .to_i64()?;
                            },
                            "f64" => quote! {
                                let #arg_name: f64 = args.get(#idx)
                                    .ok_or_else(|| format!("Missing argument {} for function {}", #idx, #fn_name_str))?
                                    .to_f64()?;
                            },
                            "String" => quote! {
                                let #arg_name: String = args.get(#idx)
                                    .ok_or_else(|| format!("Missing argument {} for function {}", #idx, #fn_name_str))?
                                    .to_string()?;
                            },
                            "Vec<i64>" => quote! {
                                let #arg_name: Vec<String> = args.get(#idx)
                                    .ok_or_else(|| format!("Missing argument {} for function {}", #idx, #fn_name_str))?
                                    .to_int_list()?;
                            },
                            "Vec<String>" => quote! {
                                let #arg_name: Vec<String> = args.get(#idx)
                                    .ok_or_else(|| format!("Missing argument {} for function {}", #idx, #fn_name_str))?
                                    .to_str_list()?;
                            },
                            _ => {
                                return syn::Error::new_spanned(
                                    ty,
                                    format!(
                                        "Unsupported argument type: {}. Use i64, String, or bool",
                                        arg_type
                                    ),
                                )
                                .to_compile_error()
                                .into();
                            },
                        };

                        arg_casts.push(cast_expr);
                        arg_names.push(arg_name);
                    }
                }

                // Validate allowed return type
                let mut ret_ty = String::new();
                match &f.sig.output {
                    ReturnType::Default => {
                        return syn::Error::new_spanned(
                            &f.sig.output,
                            "Function must return a value",
                        )
                        .to_compile_error()
                        .into();
                    },
                    ReturnType::Type(_, ty) => {
                        ret_ty = get_type(ty);
                        if !is_allowed_type(ret_ty.as_str()) {
                            return syn::Error::new_spanned(
                                ty,
                                "Return type must be bool, i64, String, Vec<i64>, or Vec<String>",
                            )
                            .to_compile_error()
                            .into();
                        }
                    },
                }

                // Generate Val constructor based on return type
                let val_constructor = match ret_ty.as_str() {
                    "bool" => quote! { Val::Bool(result.into()) },
                    "i64" => quote! { Val::Int(result.into()) },
                    "f64" => quote! { Val::Float(result.into()) },
                    "String" => quote! { Val::Str(result.into()) },
                    "Vec<i64>" => quote! { Val::IntList(result.into()) },
                    "Vec<String>" => quote! { Val::StrList(result.into()) },
                    _ => quote! { Val::Int(result.into()) }, // default
                };

                // Generate wrapper function that takes Vec<Val> and calls original
                let wrapper_fn = quote! {
                    fn #wrapper_name(args: Vec<Val>) -> RadeResult<Val> {
                        #(#arg_casts)*
                        let result = #mod_ident::#fn_name(#(#arg_names),*);
                        Ok(#val_constructor)
                    }
                };
                wrapper_fns.push(wrapper_fn);

                // Registration uses wrapper
                let val_type_ident = match ret_ty.as_str() {
                    "bool" => quote! { ValType::Bool },
                    "i64" => quote! { ValType::Int },
                    "f64" => quote! { ValType::Float },
                    "String" => quote! { ValType::String },
                    "Vec<i64>" => quote! { ValType::IntList },
                    "Vec<String>" => quote! { ValType::StringList },
                    _ => quote! { ValType::Int }, // default
                };
                registrations.push(quote! {
                    m.insert(#fn_name_str, (#wrapper_name as fn(Vec<Val>) -> RadeResult<Val>, #val_type_ident));
                });
            }
        }
    }

    // Generate final module + static map
    let expanded = quote! {
        #module

        #(#wrapper_fns)*

        struct #registry_ident(hashbrown::HashMap<&'static str, (fn(Vec<Val>) -> RadeResult<Val>, ValType)>);
        impl #registry_ident {
            fn function(&self, name: &str) -> Option<fn(Vec<Val>) -> RadeResult<Val>> {
                self.0.get(name).map(|(f, _)| *f)
            }

            fn ret_type(&self, name: &str) -> Option<&ValType> {
                self.0.get(name).map(|(_, ty)| ty)
            }
        }

        static #map_ident: spin::Lazy<#registry_ident> =
            spin::Lazy::new(|| {
                let mut m = hashbrown::HashMap::new();
                #(#registrations)*
                #registry_ident(m)
            });
    };

    TokenStream::from(expanded)
}

// Helper: allowed types
fn get_type(ty: &Type) -> String {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(seg) = path.segments.last() {
                match seg.ident.to_string().as_str() {
                    "bool" | "i64" | "f64" | "String" => seg.ident.to_string(),
                    "Vec" => {
                        if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                            if args.args.len() != 1 {
                                return String::new();
                            }
                            if let syn::GenericArgument::Type(Type::Path(TypePath { path, .. })) =
                                &args.args[0]
                                && let Some(inner) = path.segments.last()
                            {
                                return format!("Vec<{}>", inner.ident);
                            }
                        }
                        String::new()
                    },
                    _ => String::new(),
                }
            } else {
                String::new()
            }
        },
        _ => String::new(),
    }
}

fn is_allowed_type(ty: &str) -> bool {
    matches!(
        ty,
        "bool" | "i64" | "f64" | "String" | "Vec<i64>" | "Vec<String>"
    )
}
