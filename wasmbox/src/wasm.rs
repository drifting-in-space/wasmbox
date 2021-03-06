use crate::{AsyncWasmBox, AsyncWasmBoxBox, WasmBox};
use std::cell::RefCell;

extern crate alloc;

thread_local! {
    static WASM_BOX: RefCell<Option<Box<dyn WasmBox<Input = String, Output = String>>>> = RefCell::default();
}

extern "C" {
    /// Send a message from the wasm module to the host.
    pub fn wasmbox_callback(message_ptr: u32, message_len: u32);
}

pub fn wrapped_callback(message: String) {
    let message = bincode::serialize(&message).expect("Error serializing.");
    unsafe {
        wasmbox_callback(&message[0] as *const u8 as u32, message.len() as u32);
    }
}

pub fn initialize<B>()
where
    B: WasmBox<Input = String, Output = String>,
{
    let wasm_box = B::init(Box::new(wrapped_callback));
    WASM_BOX.with(|cell| cell.replace(Some(Box::new(wasm_box))));
}

pub fn initialize_async<B>()
where
    B: AsyncWasmBox<Input = String, Output = String>,
{
    let wasm_box: AsyncWasmBoxBox<B> = AsyncWasmBoxBox::init(Box::new(wrapped_callback));
    WASM_BOX.with(|cell| cell.replace(Some(Box::new(wasm_box))));
}

#[no_mangle]
extern "C" fn wasmbox_send(ptr: *const u8, len: usize) {
    let message: String = unsafe {
        let bytes = std::slice::from_raw_parts(ptr, len).to_vec();
        bincode::deserialize(&bytes).expect("Error deserializing.")
    };

    WASM_BOX.with(|cell| {
        cell.borrow_mut()
            .as_mut()
            .expect("Received message before initialized.")
            .message(message)
    });
}

#[no_mangle]
pub unsafe extern "C" fn wasmbox_malloc(size: u32) -> *mut u8 {
    let layout = core::alloc::Layout::from_size_align_unchecked(size as usize, 0);
    alloc::alloc::alloc(layout)
}

#[no_mangle]
pub unsafe extern "C" fn wasmbox_free(ptr: *mut u8, size: u32) {
    let layout = core::alloc::Layout::from_size_align_unchecked(size as usize, 0);
    alloc::alloc::dealloc(ptr, layout);
}
