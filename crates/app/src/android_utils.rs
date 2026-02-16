#[cfg(target_os = "android")]
use jni::objects::{JObject, JString, JValue};
#[cfg(target_os = "android")]
use jni::JNIEnv;

#[cfg(target_os = "android")]
pub fn copy_to_clipboard(text: &str) {
    let ctx = ndk_context::android_context();
    unsafe {
        let vm = jni::JavaVM::from_raw(ctx.vm().cast()).unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let context = JObject::from_raw(ctx.context().cast());

        // Get ClipboardManager service
        // Context.CLIPBOARD_SERVICE is "clipboard"
        let service_name = env.new_string("clipboard").unwrap();
        let clipboard = env
            .call_method(
                &context,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&service_name)],
            )
            .unwrap()
            .l()
            .unwrap();

        // Create ClipData
        // ClipData.newPlainText("label", text)
        let label = env.new_string("Construct Data").unwrap();
        let text_j = env.new_string(text).unwrap();
        
        let clip_data_class = env.find_class("android/content/ClipData").unwrap();
        let clip_data = env
            .call_static_method(
                clip_data_class,
                "newPlainText",
                "(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Landroid/content/ClipData;",
                &[JValue::Object(&label), JValue::Object(&text_j)],
            )
            .unwrap()
            .l()
            .unwrap();

        // Set primary clip
        let _ = env.call_method(
            &clipboard,
            "setPrimaryClip",
            "(Landroid/content/ClipData;)V",
            &[JValue::Object(&clip_data)],
        );
    }
}

#[cfg(target_os = "android")]
pub fn share_text(text: &str) {
    let ctx = ndk_context::android_context();
    unsafe {
        let vm = jni::JavaVM::from_raw(ctx.vm().cast()).unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let context = JObject::from_raw(ctx.context().cast());

        // Create Intent(ACTION_SEND)
        let intent_class = env.find_class("android/content/Intent").unwrap();
        let intent = env.new_object(&intent_class, "()V", &[]).unwrap();
        
        // setAction(Intent.ACTION_SEND) -> "android.intent.action.SEND"
        let action_send = env.new_string("android.intent.action.SEND").unwrap();
        let _ = env.call_method(
            &intent, 
            "setAction", 
            "(Ljava/lang/String;)Landroid/content/Intent;", 
            &[JValue::Object(&action_send)]
        ).unwrap();

        // putExtra(Intent.EXTRA_TEXT, text)
        // EXTRA_TEXT is "android.intent.extra.TEXT"
        let extra_text = env.new_string("android.intent.extra.TEXT").unwrap();
        let text_j = env.new_string(text).unwrap();
        let _ = env.call_method(
            &intent,
            "putExtra",
            "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
            &[JValue::Object(&extra_text), JValue::Object(&text_j)]
        ).unwrap();

        // setType("text/plain")
        let type_plain = env.new_string("text/plain").unwrap();
        let _ = env.call_method(
            &intent,
            "setType",
            "(Ljava/lang/String;)Landroid/content/Intent;",
            &[JValue::Object(&type_plain)]
        ).unwrap();

        // Create Chooser
        // Intent.createChooser(intent, "Share via")
        let title = env.new_string("Share Inventory").unwrap();
        let chooser = env.call_static_method(
            &intent_class,
            "createChooser",
            "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;",
            &[JValue::Object(&intent), JValue::Object(&title)]
        ).unwrap().l().unwrap();
        
        // Add FLAG_ACTIVITY_NEW_TASK just in case, though usually not needed if started from Activity context.
        // But startActivity from generic context might need it if not Activity.
        // Here we have the Activity context from ndk-context, so it should be fine.
        
        // startActivity(chooser)
        let _ = env.call_method(
            &context,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[JValue::Object(&chooser)]
        ).unwrap();
    }
}

// Dummy implementations for non-Android targets to avoid compile errors
#[cfg(not(target_os = "android"))]
pub fn copy_to_clipboard(_text: &str) {
    println!("Mock Copy: {}", _text);
}

#[cfg(not(target_os = "android"))]
pub fn share_text(_text: &str) {
    println!("Mock Share: {}", _text);
}
