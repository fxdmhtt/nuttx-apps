use std::{
    collections::{BTreeMap, HashMap},
    env, fs,
    path::Path,
};

use proc_macro2::Ident;
use quote::quote;
use serde::Deserialize;

#[derive(Deserialize)]
struct I18n {
    languages: Vec<String>,
    texts: BTreeMap<String, HashMap<String, String>>,
}

pub fn generate() {
    let input = fs::read_to_string("i18n.yaml").expect("failed to read i18n.yaml");

    let code = generate_code(&input);

    let out_dir = env::var("OUT_DIR").unwrap();
    let out = Path::new(&out_dir).join("i18n.rs");

    fs::write(&out, code).expect("failed to write i18n.rs");

    std::process::Command::new("rustfmt")
        .arg(&out)
        .status()
        .expect("failed to format i18n.rs");

    println!("cargo:rerun-if-changed=i18n.yaml");
}

fn generate_code(yaml: &str) -> String {
    let I18n { languages, texts } = serde_yaml::from_str::<I18n>(yaml).expect("failed to parse i18n.yaml");

    let lang_idents: Vec<Ident> = languages
        .iter()
        .map(|lang| syn::parse_str(lang).unwrap())
        .collect();
    let lang_enum = quote! {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Copy, Clone)]
        enum Lang {
            #(#lang_idents,)*
        }
    };

    let text_idents: Vec<Ident> = texts
        .keys()
        .map(|key| syn::parse_str(key).unwrap())
        .collect();
    let text_enum = quote! {
        #[derive(Debug, Copy, Clone)]
        enum Text {
            #(#text_idents,)*
        }
    };

    let lang_arms = languages.iter().map(|lang| {
        let text_arms = texts.iter().map(|(key, map)| {
            let text = match map.get(lang) {
                Some(s) => s,
                None => panic!("Missing translation for `{key}` in `{lang}` language."),
            };
            let key: Ident = syn::parse_str(key).unwrap();
            quote! {
                Text::#key => c #text
            }
        });

        let lang: Ident = syn::parse_str(lang).unwrap();
        quote! {
            Lang::#lang => match id { #(#text_arms,)* }
        }
    });
    let t_fn = quote! {
        #[allow(non_snake_case)]
        fn _T(lang: Lang, id: Text) -> &'static std::ffi::CStr {
            match lang { #(#lang_arms,)* }
        }
    };

    quote! {
        #lang_enum
        #text_enum
        #t_fn
    }
    .to_string()
    .replace("c \"", "c\"")
}
