//! Minimal wasm32 C ABI; JS owns input allocations until deallocation.
use crate::Chat;
use std::cell::RefCell;

thread_local! {
    static CHAT: RefCell<Chat> = RefCell::new(Chat::new(1));
    static RESPONSE: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}
fn respond(s: String) {
    RESPONSE.with(|r| *r.borrow_mut() = s.into_bytes());
}

#[no_mangle]
pub extern "C" fn chat_new(seed: u32) {
    CHAT.with(|c| {
        let chat = Chat::new(seed);
        respond(chat.start().to_json());
        *c.borrow_mut() = chat;
    });
}
#[no_mangle]
pub extern "C" fn chat_alloc(len: usize) -> *mut u8 {
    Box::into_raw(vec![0u8; len].into_boxed_slice()) as *mut u8
}
/// # Safety
/// `ptr` must be a live `chat_alloc(len)` allocation, not freed or aliased mutably.
#[no_mangle]
pub unsafe extern "C" fn chat_submit(ptr: *const u8, len: usize) {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let input = String::from_utf8_lossy(bytes);
    CHAT.with(|c| respond(c.borrow_mut().submit(&input).to_json()));
}
/// # Safety
/// Pass the exact pointer and length returned by chat_alloc, once only.
#[no_mangle]
pub unsafe extern "C" fn chat_free(ptr: *mut u8, len: usize) {
    drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)) });
}
#[no_mangle]
pub extern "C" fn chat_response_ptr() -> *const u8 {
    RESPONSE.with(|r| r.borrow().as_ptr())
}
#[no_mangle]
pub extern "C" fn chat_response_len() -> usize {
    RESPONSE.with(|r| r.borrow().len())
}
