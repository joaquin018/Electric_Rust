#[cfg(target_os = "android")]
use std::thread;

#[cfg(target_os = "android")]
use std::sync::{mpsc, RwLock};
#[cfg(target_os = "android")]
static HAPTIC_SENDER: RwLock<Option<mpsc::Sender<()>>> = RwLock::new(None);

#[cfg(target_os = "android")]
use jni::objects::{JObject, JValue};

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
                             // Try VibrationEffect first
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

#[cfg(target_os = "android")]
pub fn copy_to_clipboard(text: &str) {
    let ctx = ndk_context::android_context();
    unsafe {
        let vm = jni::JavaVM::from_raw(ctx.vm().cast()).unwrap();
        let mut env = match vm.attach_current_thread() { Ok(e) => e, Err(_) => return };
        let context = JObject::from_raw(ctx.context().cast());

        let service_name = env.new_string("clipboard").unwrap();
        let clipboard = env.call_method(&context, "getSystemService", "(Ljava/lang/String;)Ljava/lang/Object;", &[JValue::Object(&service_name)]).unwrap().l().unwrap();
        
        let label = env.new_string("Construct").unwrap();
        let text_j = env.new_string(text).unwrap();
        let clip_class = env.find_class("android/content/ClipData").unwrap();
        let clip = env.call_static_method(clip_class, "newPlainText", "(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Landroid/content/ClipData;", &[JValue::Object(&label), JValue::Object(&text_j)]).unwrap().l().unwrap();
        let _ = env.call_method(&clipboard, "setPrimaryClip", "(Landroid/content/ClipData;)V", &[JValue::Object(&clip)]);
    }
}

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

#[cfg(target_os = "android")]
pub fn get_app_files_dir() -> String {
    let ctx = ndk_context::android_context();
    unsafe {
        let vm = jni::JavaVM::from_raw(ctx.vm().cast()).unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let context = JObject::from_raw(ctx.context().cast());
        
        // File file = context.getFilesDir();
        let file = env.call_method(&context, "getFilesDir", "()Ljava/io/File;", &[]).unwrap().l().unwrap();
        
        // String path = file.getAbsolutePath();
        let path_jstr = env.call_method(&file, "getAbsolutePath", "()Ljava/lang/String;", &[]).unwrap().l().unwrap();
        
        let path: String = env.get_string(&path_jstr.into()).unwrap().into();
        path
    }
}

// Fallbacks for non-Android targets
#[cfg(not(target_os = "android"))]
pub fn init_haptics() {}

#[cfg(not(target_os = "android"))]
pub fn trigger_haptic_feedback() {}

#[cfg(not(target_os = "android"))]
pub fn copy_to_clipboard(data: &str) {
    println!("COPY: {}", data);
}

#[cfg(not(target_os = "android"))]
pub fn share_text(data: &str) {
    println!("SHARE: {}", data);
}

#[cfg(not(target_os = "android"))]
pub fn get_app_files_dir() -> String {
    ".".to_string()
}
