package com.antigravity.construct;

import android.app.NativeActivity;
import android.view.KeyEvent;

/**
 * Custom NativeActivity subclass that intercepts the Back button
 * BEFORE it reaches the native InputQueue.
 *
 * NativeActivity routes key events through native code, so standard
 * approaches (OnKeyListener, OnBackInvokedDispatcher) do not reliably
 * intercept the Back key. By overriding dispatchKeyEvent here, we
 * catch the event at the Java level first.
 */
public class BackHandler extends NativeActivity {

    // Native method — implemented in Rust via JNI
    private static native void nativeOnBackPressed();

    @Override
    public boolean dispatchKeyEvent(KeyEvent event) {
        if (event.getKeyCode() == KeyEvent.KEYCODE_BACK
                && event.getAction() == KeyEvent.ACTION_UP) {
            nativeOnBackPressed();
            return true; // consume the event — do NOT let NativeActivity finish()
        }
        return super.dispatchKeyEvent(event);
    }
}
