@echo off
:: Super-Build Minimalista para Construct Rust
set "BUILD_TOOLS=C:\android-dev\build-tools\35.0.0"
set "PLATFORM_JAR=C:\android-dev\platforms\android-36\android.jar"
set "PATH=C:\Program Files\Microsoft\jdk-17.0.18.8-hotspot\bin;C:\android-dev\platform-tools;%PATH%"

echo Compilando Rust...
call cargo apk build --package construct --release >nul

echo Corrigiendo Icono...
:: Compila recursos y los inyecta en el APK original
"%BUILD_TOOLS%\aapt.exe" package -f -M app\AndroidManifest.xml -S app\res -I "%PLATFORM_JAR%" -F resources.zip
jar uf target\release\apk\Construct.apk -C . resources.zip

echo Firmando e Instalando...
if exist Construct_Final.apk del Construct_Final.apk
"%BUILD_TOOLS%\zipalign.exe" -f 4 target\release\apk\Construct.apk Construct_Final.apk
call "%BUILD_TOOLS%\apksigner.bat" sign --ks "C:\Users\Joaquin\.android\debug.keystore" --ks-pass pass:android Construct_Final.apk
adb install -r Construct_Final.apk
adb shell am start -n com.antigravity.construct/android.app.NativeActivity

:: Limpieza
del resources.zip Construct_Final.apk
if exist *.idsig del *.idsig
