use std::{fs::File, path::Path, io::Read, ptr, str, ffi::CStr};

use crate::*;

use bson::Document;

unsafe fn with_binary_slice<T>(binary: *mut mongocrypt_binary_t, f: impl FnOnce(&[u8]) -> T) -> T {
    let data = mongocrypt_binary_data(binary);
    let len = mongocrypt_binary_len(binary);
    let slice = std::slice::from_raw_parts(data, len as usize);
    f(slice)
}

unsafe fn doc_from_binary(bytes: *mut mongocrypt_binary_t) -> Document {
    with_binary_slice(bytes, |slice| Document::from_reader(slice).unwrap())
}

struct BinaryBuffer {
    #[allow(dead_code)]
    bytes: Vec<u8>,
    binary: *mut mongocrypt_binary_t,
}

impl BinaryBuffer {
    fn read<P: AsRef<Path>>(path: P) -> std::io::Result<BinaryBuffer> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let binary = unsafe {
            let ptr = bytes.as_mut_ptr() as *mut u8;
            mongocrypt_binary_new_from_data(ptr, bytes.len() as u32)
        };
        Ok(BinaryBuffer { bytes, binary })
    }
}

impl Drop for BinaryBuffer {
    fn drop(&mut self) {
        unsafe { mongocrypt_binary_destroy(self.binary); }
    }
}

unsafe fn example_state_machine(ctx: *mut mongocrypt_ctx_t) -> Document {
    let mut result = Document::new();
    loop {
        let state = mongocrypt_ctx_state(ctx);
        match () {
            _ if state == mongocrypt_ctx_state_t_MONGOCRYPT_CTX_NEED_MONGO_COLLINFO => {
                let output = mongocrypt_binary_new();
                assert!(mongocrypt_ctx_mongo_op(ctx, output));
                println!("\nrunning listCollections on mongod with this filter:\n{:?}", doc_from_binary(output));
                mongocrypt_binary_destroy(output);
                let input = BinaryBuffer::read("test/example/collection-info.json").unwrap();
                println!("\nmocking reply from file:\n{:?}", doc_from_binary(input.binary));
                assert!(mongocrypt_ctx_mongo_feed(ctx, input.binary));
                assert!(mongocrypt_ctx_mongo_done(ctx));
            }
            _ if state == mongocrypt_ctx_state_t_MONGOCRYPT_CTX_NEED_MONGO_MARKINGS => {
                let output = mongocrypt_binary_new();
                assert!(mongocrypt_ctx_mongo_op(ctx, output));
                println!("\nrunning cmd on mongocryptd with this schema:\n{:?}", doc_from_binary(output));
                mongocrypt_binary_destroy(output);
                let input = BinaryBuffer::read("test/example/mongocryptd-reply.json").unwrap();
                println!("\nmocking reply from file:\n{:?}", doc_from_binary(input.binary));
                assert!(mongocrypt_ctx_mongo_feed(ctx, input.binary));
                assert!(mongocrypt_ctx_mongo_done(ctx));
            }
            _ if state == mongocrypt_ctx_state_t_MONGOCRYPT_CTX_NEED_MONGO_KEYS => {
                let output = mongocrypt_binary_new();
                assert!(mongocrypt_ctx_mongo_op(ctx, output));
                println!("\nrunning a find on the key vault coll with this filter:\n{:?}", doc_from_binary(output));
                mongocrypt_binary_destroy(output);
                let input = BinaryBuffer::read("test/example/key-document.json").unwrap();
                println!("\nmocking reply from file:\n{:?}", doc_from_binary(input.binary));
                assert!(mongocrypt_ctx_mongo_feed(ctx, input.binary));
                assert!(mongocrypt_ctx_mongo_done(ctx));
            }
            _ if state == mongocrypt_ctx_state_t_MONGOCRYPT_CTX_NEED_KMS => {
                loop {
                    let kms = mongocrypt_ctx_next_kms_ctx(ctx);
                    if kms == ptr::null_mut() { break; }
                    let output = mongocrypt_binary_new();
                    assert!(mongocrypt_kms_ctx_message(kms, output));
                    with_binary_slice(output, |slice| {
                        println!("sending the following to kms:\n{:?}", str::from_utf8(slice).unwrap());
                    });
                    mongocrypt_binary_destroy(output);
                    let input = BinaryBuffer::read("test/example/kms-decrypt-reply.txt").unwrap();
                    println!("mocking reply from file:\n{:?}", str::from_utf8(&input.bytes).unwrap());
                    assert!(mongocrypt_kms_ctx_feed(kms, input.binary));
                    assert_eq!(0, mongocrypt_kms_ctx_bytes_needed(kms));
                }
                mongocrypt_ctx_kms_done(ctx);
            }
            _ if state == mongocrypt_ctx_state_t_MONGOCRYPT_CTX_READY => {
                let output = mongocrypt_binary_new();
                assert!(mongocrypt_ctx_finalize(ctx, output));
                result = doc_from_binary(output);
                mongocrypt_binary_destroy(output);
            }
            _ if state == mongocrypt_ctx_state_t_MONGOCRYPT_CTX_DONE => {
                break;
            }
            _ if state == mongocrypt_ctx_state_t_MONGOCRYPT_CTX_ERROR => {
                let status = mongocrypt_status_new();
                mongocrypt_ctx_status(ctx, status);
                let message = CStr::from_ptr(mongocrypt_status_message(status, ptr::null_mut())).to_str().unwrap();
                panic!("got error: {}", message);
            }
            _ => panic!("unhandled state {:?}", state),
        }
    }

    return result;
}