use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub fn bevy_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    assert_eq!(
        input.sig.ident, "main",
        "`bevy_main` can only be used on a function called 'main'."
    );

    TokenStream::from(entry_points(&input))
}

/// The Android entry point around the app's `main`. When `main` returns, the
/// process ends: Bevy can't run again in it (see
/// `bevy::android::exit_process`).
fn entry_points(input: &ItemFn) -> proc_macro2::TokenStream {
    quote! {
        // SAFETY: `#[bevy_main]` should only be placed on a single `main` function
        // TODO: Potentially make `bevy_main` and unsafe attribute as there is a safety
        // guarantee required from the caller.
        #[unsafe(no_mangle)]
        #[cfg(target_os = "android")]
        fn android_main(android_app: bevy::android::android_activity::AndroidApp) {
            let _ = bevy::android::ANDROID_APP.set(android_app);
            main();
            bevy::android::exit_process(0);
        }

        #[allow(unused)]
        #input
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn android_main_ends_the_process_after_main_returns() {
        let main: ItemFn = syn::parse_quote! { fn main() {} };
        let file: syn::File = syn::parse2(entry_points(&main)).unwrap();
        let syn::Item::Fn(android_main) = &file.items[0] else {
            panic!("the first item is `android_main`");
        };
        assert_eq!(android_main.sig.ident, "android_main");
        let statements: Vec<String> = android_main
            .block
            .stmts
            .iter()
            .map(|s| quote!(#s).to_string())
            .collect();
        assert_eq!(
            statements[1..],
            ["main () ;", "bevy :: android :: exit_process (0) ;"]
        );
    }
}
