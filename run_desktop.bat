@echo off
setlocal
cd /d %~dp0

echo [1/1] Ejecutando en Desktop (Simulador Movil)...
:: Forzamos el backend de software o GL segun preferencia, pero por defecto cargo run
:: Usamos --package electric para asegurar que corra el binario de la carpeta app
cargo run --package electric

if %errorlevel% neq 0 (
    echo.
    echo Error: No se pudo ejecutar la aplicacion. 
    echo Asegurate de tener Rust instalado y estar en la carpeta correcta.
    pause
)
