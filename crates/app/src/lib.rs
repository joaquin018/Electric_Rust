mod android_utils;

slint::include_modules!();
use slint::ComponentHandle;
use slint::Model;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();
    main().unwrap();
}

fn main() -> Result<(), slint::PlatformError> {
    // Initialize Tokio runtime for async tasks if needed (though UI runs on main thread)
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    // Copy Handler
    let ui_weak_copy = ui_handle.clone();
    ui.on_request_copy(move || {
        if let Some(ui) = ui_weak_copy.upgrade() {
            let data = format_inventory(ui.get_inv_vals());
            #[cfg(target_os = "android")]
            android_utils::copy_to_clipboard(&data);
            #[cfg(not(target_os = "android"))]
            println!("COPY: {}", data);
        }
    });

    // Share Handler
    let ui_weak_share = ui_handle.clone();
    ui.on_request_share(move || {
        if let Some(ui) = ui_weak_share.upgrade() {
            let data = format_inventory(ui.get_inv_vals());
            #[cfg(target_os = "android")]
            android_utils::share_text(&data);
            #[cfg(not(target_os = "android"))]
            println!("SHARE: {}", data);
        }
    });

    ui.run()
}

fn format_inventory(model: slint::ModelRc<slint::SharedString>) -> String {
    let labels = [
        "Apoyos 30 cm", "Apoyos 40 cm", "Apoyos 50 cm", "Apoyos 60 cm", 
        "Apoyos 70 cm", "Apoyos 80 cm", "Apoyos 90 cm",
        "Viga 2x3\"", "Viga 2x4\"", "Viga 2x5\"", "Viga 2x6\"", "Viga 2x8\"", "Viga 2x10\"",
        "Clavos 3\"", "Clavos 3 1/2\"", "Clavos 4\"",
        "Cemento (Bolsas)"
    ];

    let mut result = String::from("INVENTARIO CONSTRUCT:\n\n");
    let mut has_content = false;

    // ModelRc isn't guaranteed to be exactly the length we expect in dynamic UI, 
    // but here it is fixed.
    let count = model.row_count();
    
    for i in 0..count {
        if i >= labels.len() { break; }
        
        // Safe access to model data
        if let Some(val) = model.row_data(i) {
            let val_str = val.as_str();
            // Filter empty or "0" values to share only relevant data?
            // Or share everything? Usually better to share non-empty.
            if !val_str.is_empty() && val_str != "0" {
                result.push_str(&format!("{}: {}\n", labels[i], val_str));
                has_content = true;
            }
        }
    }

    if !has_content {
        result.push_str("(Sin items registrados)");
    }
    
    result
}
