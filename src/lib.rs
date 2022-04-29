use std::slice;

#[no_mangle]
pub extern "C" fn IsHsuPhoEndKey(pho_inx: *const i32, key: i32) -> i32 {
    let pho_inx = unsafe { slice::from_raw_parts(pho_inx, 3) };
    match key as u8 {
        b's' | b'd' | b'f' | b'j' | b' ' => {
            if pho_inx[0] != 0 {
                pho_inx[0]
            } else if pho_inx[1] != 0 {
                pho_inx[1]
            } else {
                pho_inx[2]
            }
        }
        _ => 0,
    }
}
