package com.antigravity.construct;

import android.app.Activity;
import android.view.KeyEvent;
import android.view.View;

/**
 * Intercepts the Android Back button (KEYCODE_BACK) on the DecorView
 * and calls a native Rust function instead of letting the Activity finish.
 */
public class BackHandler implements View.OnKeyListener {

    // Native method — implemented in Rust via JNI
    private static native void nativeOnBackPressed();

    @Override
    public boolean onKey(View v, int keyCode, KeyEvent event) {
        if (keyCode == KeyEvent.KEYCODE_BACK && event.getAction() == KeyEvent.ACTION_UP) {
            nativeOnBackPressed();
            return true; // consume the event
        }
        return false; // let other keys pass through
    }

    /**
     * Called from Rust to set up the back button handler.
     * Attaches this listener to the Activity's DecorView.
     */
    public static void init(Activity activity) {
        View decorView = activity.getWindow().getDecorView();
        decorView.setOnKeyListener(new BackHandler());
        decorView.setFocusableInTouchMode(true);
        decorView.requestFocus();
    }
}
