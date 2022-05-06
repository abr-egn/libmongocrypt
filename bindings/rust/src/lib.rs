#[allow(nonstandard_style)]
mod bindings;

pub use bindings::*;

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    #[test]
    fn version() {
        let version = unsafe {
            CStr::from_ptr(crate::mongocrypt_version(std::ptr::null_mut()))
                .to_string_lossy()
                .into_owned()
        };
        assert_eq!(version, "1.3.1");
    }
}
