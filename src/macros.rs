/// Return an [`hdi::prelude::ValidateCallbackResult::Valid`]
///
/// ##### Example: Basic Usage
/// ```
/// # use hdi::prelude::*;
/// # use hdi_extensions::*;
///
/// fn pass() -> ExternResult<ValidateCallbackResult> {
///     valid!()
/// }
/// ```
#[macro_export]
macro_rules! valid {
    () => {
        return Ok($crate::hdi::prelude::ValidateCallbackResult::Valid)
    };
}

/// Return an [`hdi::prelude::ValidateCallbackResult::Invalid`]
///
/// ##### Example: Basic Usage
/// ```
/// # use hdi::prelude::*;
/// # use hdi_extensions::*;
///
/// fn fail() -> ExternResult<ValidateCallbackResult> {
///     invalid!(format!("Unauthorized"))
/// }
/// ```
#[macro_export]
macro_rules! invalid {
    ( $message:expr ) => {
        return Ok($crate::hdi::prelude::ValidateCallbackResult::Invalid($message))
    };
}

/// Shortcut for `wasm_error!(WasmErrorInner::Guest( ... ))`
///
/// ##### Example: Basic Usage
/// ```
/// # use hdi_extensions::*;
///
/// guest_error!(format!("Something's wrong"));
/// ```
#[macro_export]
macro_rules! guest_error {
    ( $message:expr ) => {
        {
            use $crate::hdi::prelude::*;
            wasm_error!(WasmErrorInner::Guest( $message ))
        }
    };
}
