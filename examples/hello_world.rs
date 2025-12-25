use std::{
    ffi::{CStr, CString, c_char, c_void},
    ptr::null_mut,
    slice::from_raw_parts,
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

fn get_entities_by_type(name: &str) -> Vec<&grug_file> {
    #[allow(static_mut_refs)]
    let mods = unsafe { grug_mods };
    let mods = unsafe { from_raw_parts(mods.dirs, mods.dirs_size) };

    let mut return_files = vec![];

    for mod_ in mods.iter() {
        let files = unsafe { from_raw_parts(mod_.files, mod_.files_size) };
        for file in files {
            let mod_entity_name = unsafe {
                CStr::from_ptr(file.entity_type)
                    .to_string_lossy()
                    .into_owned()
            };
            if mod_entity_name == name {
                return_files.push(file);
            }
        }
    }

    return_files
}

fn main() {
    unsafe {
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
            let error = grug_error;
            panic!("Grug failed to initialize {}", error.msg.to_string());
        }

        loop {
            let failed = grug_regenerate_modified_mods();

            if failed {
                #[allow(static_mut_refs)]
                let error = grug_error;
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

            for file in get_entities_by_type("World") {
                let ptr = file.on_fns as *mut unsafe extern "C" fn(*mut c_void);
                let funcs = from_raw_parts(ptr, 1); // the 1 here is because we only have one on_function
                for f in funcs {
                    f(null_mut()); // The null mut would be the arguments passed to this function
                }
            }
        }
    }
}
