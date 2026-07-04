use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::OnceLock;
use tokio::runtime::Runtime;
use typeforge_client::TypeForgeClient;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1) // Keep it light for the adapter
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime")
    })
}

#[repr(C)]
pub struct C_Prediction {
    pub text: *const c_char,
    pub score: f32,
    pub source: u32, // For simplicity: 0 = Dictionary, 1 = User, etc.
}

#[repr(C)]
pub struct C_PredictionList {
    pub predictions: *mut C_Prediction,
    pub count: usize,
    pub generation: u64,
}

pub type PredictCallback = extern "C" fn(*mut C_PredictionList, *mut libc::c_void);

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn typeforge_predict_async(
    prefix: *const c_char,
    surrounding_text: *const c_char,
    application: *const c_char,
    generation: u64,
    callback: PredictCallback,
    user_data: *mut libc::c_void,
) {
    if prefix.is_null() {
        return;
    }

    let prefix_str = unsafe {
        match CStr::from_ptr(prefix).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return,
        }
    };

    let surrounding_str = if surrounding_text.is_null() {
        String::new()
    } else {
        unsafe {
            CStr::from_ptr(surrounding_text)
                .to_str()
                .unwrap_or("")
                .to_string()
        }
    };

    let application_str = if application.is_null() {
        None
    } else {
        unsafe {
            CStr::from_ptr(application)
                .to_str()
                .ok()
                .map(|s| s.to_string())
        }
    };

    if prefix_str.is_empty() {
        return;
    }

    let callback_ptr = callback as usize;
    let user_data_ptr = user_data as usize;

    get_runtime().spawn_blocking(move || {
        let callback_fn: PredictCallback = unsafe { std::mem::transmute(callback_ptr) };
        let user_data_raw = user_data_ptr as *mut libc::c_void;

        let client = TypeForgeClient::new();
        let result = client.predict(&prefix_str, Some(&surrounding_str), 5, application_str);

        let mut c_preds = Vec::new();
        if let Ok(predictions) = result {
            for p in predictions {
                if let Ok(c_str) = CString::new(p.text) {
                    let text_ptr = c_str.into_raw();

                    let source = match p.source {
                        typeforge_client::PredictionSource::Dictionary => 0,
                        typeforge_client::PredictionSource::User => 1,
                        typeforge_client::PredictionSource::SpellCorrection => 2,
                        typeforge_client::PredictionSource::AI => 3,
                    };

                    c_preds.push(C_Prediction {
                        text: text_ptr,
                        score: p.score,
                        source,
                    });
                }
            }
        }

        let count = c_preds.len();
        let predictions_ptr = if count > 0 {
            let mut boxed_slice = c_preds.into_boxed_slice();
            let ptr = boxed_slice.as_mut_ptr();
            std::mem::forget(boxed_slice);
            ptr
        } else {
            ptr::null_mut()
        };

        let list = Box::new(C_PredictionList {
            predictions: predictions_ptr,
            count,
            generation,
        });

        callback_fn(Box::into_raw(list), user_data_raw);
    });
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn typeforge_predict_sync(
    prefix: *const c_char,
    surrounding_text: *const c_char,
    application: *const c_char,
) -> *mut C_PredictionList {
    if prefix.is_null() {
        return std::ptr::null_mut();
    }

    let prefix_str = unsafe {
        match CStr::from_ptr(prefix).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let surrounding_str = if surrounding_text.is_null() {
        String::new()
    } else {
        unsafe {
            CStr::from_ptr(surrounding_text)
                .to_str()
                .unwrap_or("")
                .to_string()
        }
    };

    let application_str = if application.is_null() {
        None
    } else {
        unsafe {
            CStr::from_ptr(application)
                .to_str()
                .ok()
                .map(|s| s.to_string())
        }
    };

    let client = TypeForgeClient::new();
    let result = client.predict(&prefix_str, Some(&surrounding_str), 5, application_str);

    let mut c_preds = Vec::new();
    if let Ok(predictions) = result {
        for p in predictions {
            if let Ok(c_str) = CString::new(p.text) {
                c_preds.push(C_Prediction {
                    text: c_str.into_raw(),
                    score: p.score,
                    source: match p.source {
                        typeforge_client::PredictionSource::Dictionary => 0,
                        typeforge_client::PredictionSource::User => 1,
                        typeforge_client::PredictionSource::SpellCorrection => 2,
                        typeforge_client::PredictionSource::AI => 3,
                    },
                });
            }
        }
    }

    let count = c_preds.len();
    let predictions_ptr = if count > 0 {
        let mut boxed_slice = c_preds.into_boxed_slice();
        let ptr = boxed_slice.as_mut_ptr();
        std::mem::forget(boxed_slice);
        ptr
    } else {
        ptr::null_mut()
    };

    Box::into_raw(Box::new(C_PredictionList {
        predictions: predictions_ptr,
        count,
        generation: 0,
    }))
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn typeforge_free_prediction_list(list: *mut C_PredictionList) {
    if list.is_null() {
        return;
    }

    unsafe {
        let list_box = Box::from_raw(list);
        if !list_box.predictions.is_null() && list_box.count > 0 {
            let slice = std::slice::from_raw_parts_mut(list_box.predictions, list_box.count);
            for p in slice {
                if !p.text.is_null() {
                    let _ = CString::from_raw(p.text as *mut c_char);
                }
            }
            let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                list_box.predictions,
                list_box.count,
            ));
        }
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn typeforge_learn(word: *const c_char, delta: i64, application: *const c_char) {
    if word.is_null() {
        return;
    }

    let word_str = unsafe {
        match CStr::from_ptr(word).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return,
        }
    };

    if word_str.is_empty() {
        return;
    }

    let application_str = if application.is_null() {
        None
    } else {
        unsafe {
            CStr::from_ptr(application)
                .to_str()
                .ok()
                .map(|s| s.to_string())
        }
    };

    // Fire and forget, no callback needed for learning
    get_runtime().spawn_blocking(move || {
        let client = TypeForgeClient::new();
        let _ = client.learn(&word_str, delta, application_str);
    });
}
