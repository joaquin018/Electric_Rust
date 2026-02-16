slint::include_modules!();

use logic::get_greeting;
use slint::ComponentHandle;

#[cfg(target_os = "android")]
use jni::objects::JValue;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    // Inicializar Slint
    slint::android::init(app).unwrap();
    main().unwrap();
}



fn main() -> Result<(), slint::PlatformError> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let ui = AppWindow::new()?;
    
    // Aquí conectaremos la lógica asíncrona en el futuro
    // Por ahora la UI es estática según los requerimientos de inventario

    ui.run()
}
