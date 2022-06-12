use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;

type AttributeArgs = syn::punctuated::Punctuated<syn::NestedMeta, syn::Token![,]>;

struct FinalConfig {}

/// Config used in case of the attribute not being able to build a valid config
const DEFAULT_ERROR_CONFIG: FinalConfig = FinalConfig {};

#[cfg(not(test))] // Work around for rust-lang/rust#62127
pub(crate) fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    // If any of the steps for this macro fail, we still want to expand to an item that is as close
    // to the expected output as possible. This helps out IDEs such that completions and other
    // related features keep working.

    // ensure that the tagged item is a function
    let input: syn::ItemFn = match syn::parse(item.clone()) {
        Ok(it) => it,
        Err(e) => return token_stream_with_error(item, e),
    };

    let config = if input.sig.ident == "main" && !input.sig.inputs.is_empty() {
        let msg = "the main function cannot accept arguments";
        Err(syn::Error::new_spanned(&input.sig.ident, msg))
    } else {
        AttributeArgs::parse_terminated
            .parse(args)
            .and_then(|args| build_config(input.clone(), args, false))
    };

    match config {
        Ok(config) => parse_knobs(input, false, config),
        Err(e) => token_stream_with_error(parse_knobs(input, false, DEFAULT_ERROR_CONFIG), e),
    }
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}

fn build_config(
    _input: syn::ItemFn,
    _args: AttributeArgs,
    _is_test: bool,
) -> Result<FinalConfig, syn::Error> {
    Ok(FinalConfig {})
}

fn parse_knobs(input: syn::ItemFn, _is_test: bool, _config: FinalConfig) -> TokenStream {
    let main_attrib = quote! {
        #[tokio::main]
    };

    let _handler_ident = quote! {
        #input.sig.ident
    };

    let new_main = quote! {
        async fn main() -> anyhow::Result<(), lambda_http::lambda_runtime::Error> {
            if atty::is(atty::Stream::Stdout) {
                // Running in a TTY: give the log output a glow-up
                tracing_subscriber::fmt()
                    .pretty()
                    .without_time()
                    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
                        .add_directive(tracing_subscriber::filter::LevelFilter::WARN.into())
                    )
                    .init();
            } else {
                // Not in a TTY: JSON formatting suitable for sending to CloudWatch Logs
                tracing_subscriber::fmt()
                    .json()
                    .without_time()
                    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
                        .add_directive(tracing_subscriber::filter::LevelFilter::WARN.into())
                    )
                    .init();
            }

            let app_ctx = crate::context::AppContext::new().await;
            tracing::debug!(?app_ctx, "AppContext created");

            lambda_runtime::run(lambda_runtime::service_fn(|evt| {
                handler(evt, &app_ctx)
            })).await?;

            Ok(())
        }
    };

    let result = quote! {
        #main_attrib
        #new_main

        #input
    };

    result.into()
}
