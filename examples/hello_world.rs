use std::{
    ffi::{CStr, CString, c_char},
    ptr::null_mut,
    str::from_utf8_unchecked,
    sync::Mutex,
};

use grug_sys::*;

unsafe extern "C" fn runtime_error_handler(
    reason: *const c_char,
    _type_: grug_runtime_error_type,
    on_fn_name: *const c_char,
    on_fn_path: *const c_char,
) {
    // Convert inputs safely
    let reason = if !reason.is_null() {
        unsafe { CStr::from_ptr(reason).to_string_lossy() }
    } else {
        "<no reason>".into()
    };

    let fn_name = if !on_fn_name.is_null() {
        unsafe { CStr::from_ptr(on_fn_name).to_string_lossy() }
    } else {
        "<unknown fn>".into()
    };

    let fn_path = if !on_fn_path.is_null() {
        unsafe { CStr::from_ptr(on_fn_path).to_string_lossy() }
    } else {
        "<unknown path>".into()
    };

    // Printing is OK — no panics, no unwinding
    eprintln!(
        "Grug runtime error: {}\n  at {} ({})",
        reason, fn_name, fn_path
    );
}

#[unsafe(no_mangle)]
unsafe extern "C" fn game_fn_println(message: *const c_char) {
    if message.is_null() {
        return;
    }

    let cstr = unsafe { CStr::from_ptr(message) };
    println!("{}", cstr.to_string_lossy());
}

trait ToStringWrapper {
    fn to_string(&self) -> String;
}

impl<const S: usize> ToStringWrapper for [i8; S] {
    fn to_string(&self) -> String
    where
        Self: Clone,
    {
        String::from_utf8(self.clone().to_vec().into_iter().map(|x| x as u8).collect()).unwrap()
    }
}

fn main() {
    let path = std::env::current_dir().unwrap();
    println!("The current directory is {}", path.display());

    unsafe {
        // Keep CStrings alive for the duration of the call
        let mod_api = CString::new("./examples/mod_api.json").unwrap();
        let mods = CString::new("./examples/mods").unwrap();
        let dlls = CString::new("./examples/mods_dll").unwrap();

        let failed = grug_init(
            Some(runtime_error_handler),
            mod_api.as_ptr(),
            mods.as_ptr(),
            dlls.as_ptr(),
            10_000,
        );

        if failed {
            #[allow(static_mut_refs)]
            let error = grug_error.clone();
            panic!("Grug failed to initialize {}", error.msg.to_string());
        }

        grug_set_on_fns_to_safe_mode();

        loop {
            let failed = grug_regenerate_modified_mods();

            if failed {
                #[allow(static_mut_refs)]
                let error = grug_error.clone();
                if error.has_changed {
                    if grug_loading_error_in_grug_file {
                        eprintln!(
                            "Grug failed to initialize {} {} {}",
                            error.msg.to_string(),
                            error.path.to_string(),
                            error.grug_c_line_number
                        );
                    } else {
                        eprintln!(
                            "Grug failed to initialize {} {}",
                            error.msg.to_string(),
                            error.grug_c_line_number
                        );
                    }
                }

                continue;
            }
            println!("Successfully loaded");

            let entity = CString::new("WorldEntity").unwrap();
            let file = grug_get_entity_file(entity.as_ptr());

            #[allow(static_mut_refs)]
            let mods = grug_mods.clone();
            println!("{:#?}", mods);

            if file == null_mut() {
                panic!("Entity file not found");
            }
        }

        // use `file` here
    }
}
