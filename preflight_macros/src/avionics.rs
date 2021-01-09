use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Error, ItemImpl, Result};

use crate::util::reconstruct;

#[derive(Debug, FromMeta)]
pub struct AvionicsParameters {
    default: String,
    #[darling(default)]
    no_panic: bool,
}

pub fn harness(params: AvionicsParameters, input: ItemImpl) -> Result<TokenStream> {
    let preflight_testing = option_env!("__PREFLIGHT") == Some("testing");

    let (implementation, st) = {
        let ItemImpl {
            self_ty, trait_, ..
        } = &input;

        let (invert, trait_, _) = &trait_
            .as_ref()
            .ok_or_else(|| Error::new(input.span(), "no trait was found to implement"))?;

        let trait_str = {
            let mut trait_str = if trait_.leading_colon.is_some() {
                "::".to_string()
            } else {
                String::new()
            };

            trait_str.push_str(&reconstruct(&trait_.segments));

            trait_str
        };

        if !trait_str.ends_with("Avionics") {
            return Err(Error::new(
                trait_.span(),
                "expected a trait implementation of `Avionics`",
            ));
        }

        if let Some(invert) = invert {
            return Err(Error::new(
                invert.span(),
                "cannot negate the `Avionics` implementation",
            ));
        }

        (&input, self_ty)
    };

    let platform_impl = if preflight_testing {
        // Running under preflight

        quote! {
            #[no_mangle]
            pub unsafe fn avionics_guide(sensors: &Sensors) -> Option<Control> {
                AVIONICS.guide(sensors)
            }

            #[no_mangle]
            pub static __PREFLIGHT: bool = true;

            type PanicCallback = fn(panic_info: &core::panic::PanicInfo, avionics_state: &dyn Avionics);

            static mut __PANIC_CALLBACK: Option<PanicCallback> = None;

            #[no_mangle]
            pub unsafe fn set_panic_callback(callback: PanicCallback) -> Option<PanicCallback> {
                __PANIC_CALLBACK.replace(callback)
            }
        }
    } else {
        // Building

        quote! {
            #[no_mangle]
            unsafe extern "C" fn avionics_guide(sensors: &Sensors) -> Option<Control> {
                AVIONICS.guide(sensors)
            }
        }
    };

    let default = {
        let default: TokenStream = params.default.parse()?;

        quote_spanned! {params.default.span()=>
            #default
        }
    };

    let platform_panic_handler = if preflight_testing {
        quote! {
            if let Some(callback) = unsafe { __PANIC_CALLBACK } {
                callback(_panic_info, unsafe { &AVIONICS })
            }
        }
    } else {
        quote! {
            extern "C" {
                fn panic_abort();
            }

            unsafe { panic_abort() };
        }
    };

    let panic_handler = if params.no_panic {
        quote! {}
    } else {
        quote! {
            // TODO: PUT uC IN DEEP SLEEP ON PANIC OR SMTHN or call back into c code to handle panic
            #[cfg_attr(not(test), panic_handler)]
            fn handle_panic(_panic_info: &core::panic::PanicInfo) -> ! {
                #platform_panic_handler

                loop {
                    core::sync::atomic::spin_loop_hint()
                }
            }
        }
    };

    Ok(quote! {
        #implementation

        #[doc(hidden)]
        mod __PREFLIGHT {
            use super::*;

            static mut AVIONICS: #st = #default();

            #platform_impl

            #panic_handler
        }
    })
}