package com.shifra.language;

import android.app.Activity;
import android.content.Intent;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.view.View;
import android.view.animation.AlphaAnimation;

public class SplashActivity extends Activity {

    private static final long MIN_SPLASH_MS = 1600;

    private final Handler handler = new Handler(Looper.getMainLooper());
    private final Runnable go = () -> {
        startActivity(new Intent(this, MainActivity.class));
        finish();
    };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_splash);
        View root = findViewById(R.id.splash_root);
        AlphaAnimation fade = new AlphaAnimation(0f, 1f);
        fade.setDuration(450);
        root.startAnimation(fade);
        handler.postDelayed(go, MIN_SPLASH_MS);
    }

    @Override
    protected void onDestroy() {
        handler.removeCallbacks(go);
        super.onDestroy();
    }
}