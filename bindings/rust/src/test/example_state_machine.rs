use std::{fs::File, path::Path, io::Read};

use crate::*;

use bson::{Bson, Document, raw::RawDocument, Binary};

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

unsafe fn example_state_machine(ctx: *mut mongocrypt_ctx_t) -> Bson {
    let status = mongocrypt_status_new();

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
            _ => break,
        }
    }

    todo!()
}