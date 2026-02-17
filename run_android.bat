@echo off
setlocal enabledelayedexpansion

:: --- CONFIGURACIÓN ---
set "BUILD_TOOLS=C:\android-dev\build-tools\35.0.0"
set "PLATFORM_JAR=C:\android-dev\platforms\android-36\android.jar"
set "KEYSTORE=C:\Users\Joaquin\.android\debug.keystore"
set "PATH=C:\Program Files\Microsoft\jdk-17.0.18.8-hotspot\bin;C:\android-dev\platform-tools;%PATH%"

echo.
echo [0/4] Limpiando archivos antiguos...
:: Borramos cualquier APK o IDSIG que ande suelto en la raiz
if exist *.apk del /q *.apk
if exist *.idsig del /q *.idsig
if exist temp_build rmdir /s /q temp_build

echo [1/4] Compilando Rust...
call cargo apk build --package construct --release >nul

echo [2/4] Generando Base de Recursos (Metodo Google)...
"%BUILD_TOOLS%\aapt.exe" package -f -M app\AndroidManifest.xml -S app\res -I "%PLATFORM_JAR%" -F resources_base.apk

echo [3/4] Fusionando Codigo Rust y Recursos...
mkdir temp_build
cd temp_build
jar xf ..\resources_base.apk
jar xf ..\target\release\apk\Construct.apk lib/
if exist META-INF rmdir /s /q META-INF
jar cf0 ..\Construct_Unsigned.apk .
cd ..

echo [4/4] Firmando y Desplegando...
"%BUILD_TOOLS%\zipalign.exe" -f 4 Construct_Unsigned.apk Construct_Aligned.apk
:: Firmamos (esto genera el .apk y a veces un .idsig)
call "%BUILD_TOOLS%\apksigner.bat" sign --ks "%KEYSTORE%" --ks-pass pass:android --key-pass pass:android --out Construct_Final.apk Construct_Aligned.apk

echo.
echo [INFO] Instalando en el dispositivo...
adb install -r Construct_Final.apk
adb shell am start -n com.antigravity.construct/android.app.NativeActivity

echo.
echo [5/5] Limpieza final de temporales...
:: Borramos TODO lo generado en la raiz para no acumular basura
del /q resources_base.apk
del /q Construct_Unsigned.apk
del /q Construct_Aligned.apk
del /q Construct_Final.apk
if exist *.idsig del /q *.idsig
rmdir /s /q temp_build

echo.
echo ==========================================
echo  SOLUCION COMPLETA Y CARPETA LIMPIA
echo ==========================================
pause
