#[allow(nonstandard_style)]
mod bindings;

pub use bindings::*;

#[cfg(test)]
mod tests {
    use std::{ffi::CStr, ptr};

    use crate::mongocrypt_binary_destroy;

    #[test]
    fn version_is_utf8() {
        let c_version = unsafe {
            CStr::from_ptr(crate::mongocrypt_version(ptr::null_mut()))
        };
        let version = c_version.to_str();
        assert!(version.is_ok(), "{}", version.unwrap_err());
    }

    #[test]
    fn binary_empty() {
        unsafe {
            let bin = crate::mongocrypt_binary_new();
            assert_eq!(ptr::null_mut(), crate::mongocrypt_binary_data(bin));
            mongocrypt_binary_destroy(bin);
        }
    }

    #[test]
    fn binary_roundtrip() {
        let mut data = [1, 2, 3];
        unsafe {
            let data_ptr = data.as_mut_ptr() as *mut u8;
            let bin = crate::mongocrypt_binary_new_from_data(data_ptr, data.len() as u32);
            assert_eq!(crate::mongocrypt_binary_data(bin), data_ptr);
            assert_eq!(crate::mongocrypt_binary_len(bin), data.len() as u32);
            mongocrypt_binary_destroy(bin);
        }
    }

    #[test]
    fn status_roundtrip() {
        let message = CStr::from_bytes_with_nul(b"hello mongocryptd\0").unwrap();
        unsafe {
            let status = crate::mongocrypt_status_new();
            crate::mongocrypt_status_set(
                status,
                crate::mongocrypt_status_type_t_MONGOCRYPT_STATUS_ERROR_CLIENT,
                42,
                message.as_ptr(),
                -1
            );
            assert_eq!(
                crate::mongocrypt_status_type(status),
                crate::mongocrypt_status_type_t_MONGOCRYPT_STATUS_ERROR_CLIENT,
            );
            assert_eq!(
                crate::mongocrypt_status_code(status),
                42,
            );
            assert_eq!(
                CStr::from_ptr(crate::mongocrypt_status_message(status, ptr::null_mut())),
                message,
            );
        }
    }
}
