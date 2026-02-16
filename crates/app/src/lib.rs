mod android_utils;

slint::include_modules!();
use slint::ComponentHandle;
use slint::Model;
use std::time::Duration;

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
            
            // Show toast
            ui.set_show_copy_toast(true);
            
            // Auto-hide after 2.5s using thread (simpler than keeping Timer alive)
            let ui_weak_thread = ui_weak_copy.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(2500));
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak_thread.upgrade() {
                        ui.set_show_copy_toast(false);
                    }
                }).unwrap();
            });
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
    struct Section<'a> {
        header: &'a str,
        range: std::ops::Range<usize>,
        labels: &'a [&'a str],
    }

    let sections = [
        Section {
            header: "Apoyos",
            range: 0..7,
            labels: &[
                "30 cm", "40 cm", "50 cm", "60 cm", 
                "70 cm", "80 cm", "90 cm"
            ],
        },
        Section {
            header: "Vigas y Madera",
            range: 7..13,
            labels: &[
                "2x3\"", "2x4\"", "2x5\"", "2x6\"", 
                "2x8\"", "2x10\""
            ],
        },
        Section {
            header: "Clavos",
            range: 13..16,
            labels: &[
                "3\"", "3 1/2\"", "4\""
            ],
        },
        Section {
            header: "Cemento",
            range: 16..17,
            labels: &["Bolsas"],
        },
    ];

    let mut result = String::new();
    let mut is_first_section = true;

    for section in sections.iter() {
        let mut section_lines = Vec::new();

        // Collect valid items in this section
        for (i, label) in section.range.clone().zip(section.labels.iter()) {
            if let Some(val) = model.row_data(i) {
                let val_str = val.as_str();
                // Check if value is numeric and > 0
                // We treat anything not empty and not "0" as valid.
                if !val_str.is_empty() && val_str != "0" {
                    // Special formatting for Cemento
                    if section.header == "Cemento" {
                         section_lines.push(format!("- {} bolsas", val_str));
                    } else {
                         section_lines.push(format!("- {} de {}", val_str, label));
                    }
                }
            }
        }

        // If we found items, append header and lines
        if !section_lines.is_empty() {
            if !is_first_section {
                result.push_str("\n");
            }
            result.push_str(section.header);
            result.push_str("\n");
            
            for line in section_lines {
                result.push_str(&line);
                result.push_str("\n");
            }
            is_first_section = false;
        }
    }

    if result.is_empty() {
        return String::from("(Sin items seleccionados)");
    }
    
    result
}
