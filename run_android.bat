@echo off
setlocal enabledelayedexpansion

:: --- CONFIGURACIÓN ---
set "BUILD_TOOLS=C:\android-dev\build-tools\35.0.0"
set "PLATFORM_JAR=C:\android-dev\platforms\android-36\android.jar"
set "KEYSTORE=C:\Users\Joaquin\.android\debug.keystore"
set "PATH=C:\Program Files\Microsoft\jdk-17.0.18.8-hotspot\bin;C:\android-dev\platform-tools;%PATH%"

echo [1/4] Compilando Rust...
call cargo apk build --package construct --release >nul

echo [2/4] Generando Base de Recursos (Metodo Google)...
:: AAPT genera el AndroidManifest binario y la tabla de recursos correcta
if exist resources_base.apk del resources_base.apk
"%BUILD_TOOLS%\aapt.exe" package -f -M app\AndroidManifest.xml -S app\res -I "%PLATFORM_JAR%" -F resources_base.apk

echo [3/4] Fusionando Codigo Rust y Recursos...
if exist temp_build rmdir /s /q temp_build
mkdir temp_build
cd temp_build

:: Extraemos los recursos compilados (Base limpia)
jar xf ..\resources_base.apk
:: Extraemos solo las librerias (.so) del APK de Rust
jar xf ..\target\release\apk\Construct.apk lib/

:: Borramos rastros de firmas anteriores
if exist META-INF rmdir /s /q META-INF

:: Empaquetamos todo de nuevo
jar cf0 ..\Construct_Unsigned.apk .
cd ..

echo [4/4] Firmando y Desplegando...
if exist Construct_Final.apk del Construct_Final.apk
"%BUILD_TOOLS%\zipalign.exe" -f 4 Construct_Unsigned.apk Construct_Aligned.apk
call "%BUILD_TOOLS%\apksigner.bat" sign --ks "%KEYSTORE%" --ks-pass pass:android --key-pass pass:android --out Construct_Final.apk Construct_Aligned.apk

adb install -r Construct_Final.apk
adb shell am start -n com.antigravity.construct/android.app.NativeActivity

:: Limpieza
del resources_base.apk Construct_Unsigned.apk Construct_Aligned.apk
rmdir /s /q temp_build
echo.
echo ==========================================
echo  SOLUCION AISLADA APLICADA CON EXITO
echo ==========================================
pause
