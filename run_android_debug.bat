@echo off
setlocal enabledelayedexpansion
cd /d %~dp0

:: --- CONFIGURACIÓN ---
set "BUILD_TOOLS=C:\android-dev\build-tools\35.0.0"
set "PLATFORM_JAR=C:\android-dev\platforms\android-36\android.jar"
set "KEYSTORE=C:\Users\Joaquin\.android\debug.keystore"
set "PATH=C:\Program Files\Microsoft\jdk-17.0.18.8-hotspot\bin;C:\android-dev\platform-tools;%PATH%"

echo [1/5] Compilando Rust (DEBUG)...
call cargo apk build --package construct >nul

echo [2/5] Compilando Java (BackHandler)...
if exist java_build rmdir /s /q java_build
mkdir java_build
javac --release 8 -cp "%PLATFORM_JAR%" -d java_build app\java\com\antigravity\construct\BackHandler.java
call "%BUILD_TOOLS%\d8.bat" --lib "%PLATFORM_JAR%" --output java_build java_build\com\antigravity\construct\BackHandler.class

echo [3/5] Generando Base de Recursos (Metodo Google)...
:: AAPT genera el AndroidManifest binario y la tabla de recursos correcta
if exist resources_base.apk del resources_base.apk
"%BUILD_TOOLS%\aapt.exe" package -f -M app\AndroidManifest.xml -S app\res -I "%PLATFORM_JAR%" -F resources_base.apk

echo [4/5] Fusionando Codigo Rust, Java y Recursos...
if exist temp_build rmdir /s /q temp_build
mkdir temp_build
cd temp_build

:: Extraemos los recursos compilados (Base limpia)
jar xf ..\resources_base.apk
:: Extraemos solo las librerias (.so) del APK de Rust (DEBUG path)
jar xf ..\target\debug\apk\Construct.apk lib/
:: Copiamos el classes.dex generado por d8
copy ..\java_build\classes.dex . >nul

:: Borramos rastros de firmas anteriores
if exist META-INF rmdir /s /q META-INF

:: Empaquetamos todo de nuevo
jar cf0 ..\Construct_Unsigned.apk .
cd ..

echo [5/5] Firmando y Desplegando...
if exist Construct_Final.apk del Construct_Final.apk
"%BUILD_TOOLS%\zipalign.exe" -f 4 Construct_Unsigned.apk Construct_Aligned.apk
call "%BUILD_TOOLS%\apksigner.bat" sign --ks "%KEYSTORE%" --ks-pass pass:android --key-pass pass:android --out Construct_Final.apk Construct_Aligned.apk

adb install -r Construct_Final.apk
adb shell am start -n com.antigravity.construct/com.antigravity.construct.BackHandler

:: Limpieza
del resources_base.apk Construct_Unsigned.apk Construct_Aligned.apk Construct_Final.apk Construct_Final.apk.idsig
if exist Construct.apk del Construct.apk
if exist Construct.apk.idsig del Construct.apk.idsig
rmdir /s /q temp_build
rmdir /s /q java_build
echo.
echo ==========================================
echo  DEBUG BUILD INSTALLED SUCCESSFULLY
echo ==========================================
pause
