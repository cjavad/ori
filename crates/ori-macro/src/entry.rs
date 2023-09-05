#[allow(unused_imports)]
use quote::{quote, ToTokens};

pub fn main(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<syn::ItemFn>(input)?;

    let vis = &input.vis;
    let attrs = &input.attrs;
    let sig = &input.sig;
    let body = &input.block;
    let winit = crate::find_winit();

    let expanded = quote! {
        #[no_mangle]
        #[cfg(target_os = "android")]
        fn android_main(app: #winit::__private::AndroidApp) {
            #winit::__private::set_android_app(app);

            let body = || #body;
            body();
        }

        #[allow(dead_code, clippy::needless_return)]
        #(#attrs)*
        #vis #sig {
            #[warn(dead_code, clippy::needless_return)]
            #body
        }
    };

    Ok(expanded.into())
}
