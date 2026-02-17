#[cfg(target_os = "android")]
use std::thread;

#[cfg(target_os = "android")]
use std::sync::{mpsc, RwLock};
#[cfg(target_os = "android")]
static HAPTIC_SENDER: RwLock<Option<mpsc::Sender<()>>> = RwLock::new(None);

#[cfg(target_os = "android")]
use jni::objects::{JObject, JValue};

// ── Back Button ─────────────────────────────────────────────
use std::sync::atomic::{AtomicBool, Ordering};
static BACK_PRESSED: AtomicBool = AtomicBool::new(false);

/// Called from BackHandler.java when the user presses the Android Back button.
/// This is a JNI native method.
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_antigravity_construct_BackHandler_nativeOnBackPressed(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
) {
    BACK_PRESSED.store(true, Ordering::SeqCst);
}

/// Check if back was pressed and atomically reset the flag.
pub fn check_back_pressed() -> bool {
    BACK_PRESSED.swap(false, Ordering::SeqCst)
}

/// Register the Java BackHandler on the Activity's DecorView.
/// Must be called on a thread attached to the JVM.
#[cfg(target_os = "android")]
pub fn register_back_handler() {
    let ctx = ndk_context::android_context();
    unsafe {
        let vm = match jni::JavaVM::from_raw(ctx.vm().cast()) {
            Ok(vm) => vm,
            Err(_) => return,
        };
        let mut env = match vm.attach_current_thread() {
            Ok(e) => e,
            Err(_) => return,
        };
        let activity = JObject::from_raw(ctx.context().cast());

        // Register native method for BackHandler class
        let back_handler_class = match env.find_class("com/antigravity/construct/BackHandler") {
            Ok(c) => c,
            Err(_) => return, // Class not found — DEX not included in APK
        };

        // Register the native method so JVM knows where nativeOnBackPressed points
        let native_methods = [jni::NativeMethod {
            name: "nativeOnBackPressed".into(),
            sig: "()V".into(),
            fn_ptr: Java_com_antigravity_construct_BackHandler_nativeOnBackPressed as *mut std::ffi::c_void,
        }];

        if env.register_native_methods(&back_handler_class, &native_methods).is_err() {
            return;
        }

        // Call BackHandler.init(activity) — this runs on the current thread 
        // which should be fine since we call it early during app startup
        let _ = env.call_static_method(
            &back_handler_class,
            "init",
            "(Landroid/app/Activity;)V",
            &[JValue::Object(&activity)],
        );
    }
}

#[cfg(not(target_os = "android"))]
pub fn register_back_handler() {}

// ── Haptics ─────────────────────────────────────────────────

#[cfg(target_os = "android")]
pub fn init_haptics() {
    if HAPTIC_SENDER.read().unwrap().is_some() { return; }

    let (tx, rx) = mpsc::channel::<()>();
    if let Ok(mut sender_guard) = HAPTIC_SENDER.write() {
        *sender_guard = Some(tx);
    }

    thread::spawn(move || {
        let ctx = ndk_context::android_context();
        unsafe {
            let vm = match jni::JavaVM::from_raw(ctx.vm().cast()) {
                Ok(vm) => vm,
                Err(_) => return,
            };
            
            let mut env = match vm.attach_current_thread() {
                Ok(env) => env,
                Err(_) => return,
            };
            
            let context = JObject::from_raw(ctx.context().cast());
            let service_name = match env.new_string("vibrator") {
                Ok(s) => s,
                Err(_) => return,
            };
            
            let vibrator_obj = match env.call_method(
                &context,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&service_name)],
            ) {
                Ok(v) => match v.l() { Ok(o) => o, Err(_) => return },
                Err(_) => return,
            };
            
            let vibrator_global = env.new_global_ref(vibrator_obj).unwrap();

            while let Ok(_) = rx.recv() {
                let _: jni::errors::Result<JObject> = env.with_local_frame(16, |env| {
                    if let Ok(has_vibrator) = env.call_method(&vibrator_global, "hasVibrator", "()Z", &[]) {
                         if has_vibrator.z().unwrap_or(false) {
                             let effect = if let Ok(effect_class) = env.find_class("android/os/VibrationEffect") {
                                 env.call_static_method(
                                     effect_class,
                                     "createOneShot",
                                     "(JI)Landroid/os/VibrationEffect;",
                                     &[JValue::Long(5), JValue::Int(-1)]
                                 ).map(|v| v.l().ok()).ok().flatten()
                             } else {
                                 None
                             };

                             if let Some(effect_obj) = effect {
                                 let _ = env.call_method(
                                     &vibrator_global,
                                     "vibrate",
                                     "(Landroid/os/VibrationEffect;)V",
                                     &[JValue::Object(&effect_obj)]
                                 );
                             } else {
                                 let _ = env.call_method(&vibrator_global, "vibrate", "(J)V", &[JValue::Long(5)]);
                             }
                         }
                    }
                    Ok(JObject::null())
                });
            }
        }
    });
}

#[cfg(target_os = "android")]
pub fn trigger_haptic_feedback() {
    if let Ok(guard) = HAPTIC_SENDER.read() {
        if let Some(tx) = guard.as_ref() {
            let _ = tx.send(());
        }
    }
}

// ── Share ───────────────────────────────────────────────────

#[cfg(target_os = "android")]
pub fn share_text(text: &str) {
    let ctx = ndk_context::android_context();
    unsafe {
       let vm = jni::JavaVM::from_raw(ctx.vm().cast()).unwrap();
       let mut env = match vm.attach_current_thread() { Ok(e) => e, Err(_) => return };
       let context = JObject::from_raw(ctx.context().cast());
       
       let intent_class = env.find_class("android/content/Intent").unwrap();
       let intent = env.new_object(&intent_class, "()V", &[]).unwrap();
       let action = env.new_string("android.intent.action.SEND").unwrap();
       let _ = env.call_method(&intent, "setAction", "(Ljava/lang/String;)Landroid/content/Intent;", &[JValue::Object(&action)]);
       
       let extra = env.new_string("android.intent.extra.TEXT").unwrap();
       let val = env.new_string(text).unwrap();
       let _ = env.call_method(&intent, "putExtra", "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;", &[JValue::Object(&extra), JValue::Object(&val)]);
       
       let type_s = env.new_string("text/plain").unwrap();
       let _ = env.call_method(&intent, "setType", "(Ljava/lang/String;)Landroid/content/Intent;", &[JValue::Object(&type_s)]);
       
       let title = env.new_string("Share").unwrap();
       let chooser = env.call_static_method(&intent_class, "createChooser", "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;", &[JValue::Object(&intent), JValue::Object(&title)]).unwrap().l().unwrap();
       
       let _ = env.call_method(&context, "startActivity", "(Landroid/content/Intent;)V", &[JValue::Object(&chooser)]);
    }
}

// ── Storage ─────────────────────────────────────────────────

#[cfg(target_os = "android")]
pub fn get_app_files_dir() -> String {
    let ctx = ndk_context::android_context();
    unsafe {
        let vm = jni::JavaVM::from_raw(ctx.vm().cast()).unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let context = JObject::from_raw(ctx.context().cast());
        
        let file = env.call_method(&context, "getFilesDir", "()Ljava/io/File;", &[]).unwrap().l().unwrap();
        let path_jstr = env.call_method(&file, "getAbsolutePath", "()Ljava/lang/String;", &[]).unwrap().l().unwrap();
        
        let path: String = env.get_string(&path_jstr.into()).unwrap().into();
        path
    }
}

// ── System Bars ─────────────────────────────────────────────

#[cfg(target_os = "android")]
pub fn get_system_bar_bottom() -> i32 {
    let ctx = ndk_context::android_context();
    unsafe {
        let vm = jni::JavaVM::from_raw(ctx.vm().cast()).unwrap();
        let mut env = match vm.attach_current_thread() { Ok(e) => e, Err(_) => return 0 };
        let context = JObject::from_raw(ctx.context().cast());
        
        let window = match env.call_method(&context, "getWindow", "()Landroid/view/Window;", &[]) {
            Ok(v) => match v.l() { Ok(o) => o, Err(_) => return 0 },
            Err(_) => return 0,
        };
        
        let decor_view = match env.call_method(&window, "getDecorView", "()Landroid/view/View;", &[]) {
            Ok(v) => match v.l() { Ok(o) => o, Err(_) => return 0 },
            Err(_) => return 0,
        };
        
        let insets = match env.call_method(&decor_view, "getRootWindowInsets", "()Landroid/view/WindowInsets;", &[]) {
            Ok(v) => match v.l() { Ok(o) => o, Err(_) => return 0 },
            Err(_) => return 0,
        };
        
        if insets.is_null() {
             return 0;
        }

        let bottom = match env.call_method(&insets, "getSystemWindowInsetBottom", "()I", &[]) {
             Ok(v) => v.i().unwrap_or(0),
             Err(_) => 0,
        };
        
        bottom
    }
}

// ── Non-Android Fallbacks ───────────────────────────────────

#[cfg(not(target_os = "android"))]
pub fn init_haptics() {}

#[cfg(not(target_os = "android"))]
pub fn trigger_haptic_feedback() {}

#[cfg(not(target_os = "android"))]
pub fn share_text(data: &str) {
    println!("SHARE: {}", data);
}

#[cfg(not(target_os = "android"))]
pub fn get_app_files_dir() -> String {
    ".".to_string()
}

#[cfg(not(target_os = "android"))]
pub fn get_system_bar_bottom() -> i32 {
    0
}
