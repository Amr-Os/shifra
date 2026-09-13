package com.shifra.language;

import android.app.Activity;
import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Intent;
import android.content.pm.ApplicationInfo;
import android.os.Build;
import android.os.Bundle;
import android.view.WindowManager;
import android.text.method.ScrollingMovementMethod;
import android.widget.EditText;
import android.widget.TextView;
import android.widget.Toast;

import java.io.BufferedReader;
import java.io.BufferedInputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.util.ArrayDeque;

public class OutputActivity extends Activity {

    private TextView output;
    private TextView meta;
    private Process proc;
    private final ArrayDeque<String> outLines = new ArrayDeque<>();
    private final StringBuilder outPartial = new StringBuilder();
    private boolean uiPending;
    private static final int MAX_OUTPUT_LINES = 400;
    private int exit = -1;
    private long elapsedMs;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        getWindow().setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE
                | WindowManager.LayoutParams.SOFT_INPUT_STATE_ALWAYS_HIDDEN);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            getWindow().getDecorView().setOnApplyWindowInsetsListener((v, insets) -> {
                int ime = insets.getInsets(android.view.WindowInsets.Type.ime()).bottom;
                findViewById(android.R.id.content).setPadding(0, 0, 0, ime);
                return insets;
            });
        }
        setContentView(R.layout.output);
        MainActivity.enableImmersive(this);

        output = findViewById(R.id.output);
        output.setMovementMethod(new ScrollingMovementMethod());
        meta = findViewById(R.id.meta);

        final String source = getIntent().getStringExtra(Intent.EXTRA_TEXT);
        final String title = getIntent().getStringExtra(Intent.EXTRA_TITLE);
        output.setText(getString(R.string.running));

        findViewById(R.id.bt_back).setOnClickListener(v -> finish());
        findViewById(R.id.bt_copy).setOnClickListener(v -> copyOutput());

        EditText inField = findViewById(R.id.in_field);
        findViewById(R.id.in_send).setOnClickListener(v -> {
            sendStdInput(inField.getText().toString());
            inField.setText("");
        });
        inField.setOnEditorActionListener((v, actionId, ev) -> {
            sendStdInput(inField.getText().toString());
            inField.setText("");
            return true;
        });

        final ApplicationInfo info = getApplicationInfo();
        new Thread(() -> {
            long start = System.currentTimeMillis();
            try {
                File script = new File(getCacheDir(), "run.sf");
                FileOutputStream fos = new FileOutputStream(script);
                fos.write(source.getBytes(StandardCharsets.UTF_8));
                fos.flush();
                fos.close();

                File binary = new File(info.nativeLibraryDir, "libshifra.so");
                Process p =
                        new ProcessBuilder(binary.getAbsolutePath(), script.getAbsolutePath())
                                .directory(getCacheDir())
                                .redirectErrorStream(true)
                                .start();
                proc = p;
                BufferedInputStream bin = new BufferedInputStream(p.getInputStream(), 8192);
                byte[] buf = new byte[4096];
                int n;
                while ((n = bin.read(buf)) > 0) {
                    synchronized (outLines) {
                        outPartial.append(new String(buf, 0, n, StandardCharsets.UTF_8));
                        int nl;
                        while ((nl = outPartial.indexOf("\n")) >= 0) {
                            String full = outPartial.substring(0, nl);
                            outPartial.delete(0, nl + 1);
                            outLines.addLast(full);
                            while (outLines.size() > MAX_OUTPUT_LINES) outLines.removeFirst();
                        }
                    }
                    flushOutputUi();
                }
                synchronized (outLines) {
                    if (outPartial.length() > 0) {
                        outLines.addLast(outPartial.toString());
                        outPartial.setLength(0);
                    }
                }
                bin.close();
                exit = p.waitFor();
                proc = null;
            } catch (Exception e) {
                final String msg = e.getMessage();
                runOnUiThread(() -> output.setText("خطأ: " + msg));
            }
            elapsedMs = System.currentTimeMillis() - start;
            final long ms = elapsedMs;
            final int e = exit;
            if (e != 0) commitLine("[" + getString(R.string.exit_code, e) + "]");
            runOnUiThread(() -> meta.setText(title + " — " + (ms / 1000.0) + " " + getString(R.string.elapsed)));
        }).start();
    }

    private void commitLine(String line) {
        synchronized (outLines) {
            outLines.addLast(line);
            while (outLines.size() > MAX_OUTPUT_LINES) outLines.removeFirst();
        }
        flushOutputUi();
    }

    private void flushOutputUi() {
        if (uiPending) return;
        uiPending = true;
        runOnUiThread(() -> {
            uiPending = false;
            StringBuilder s = new StringBuilder();
            synchronized (outLines) {
                for (String l : outLines) s.append(l).append('\n');
                s.append(outPartial);
            }
            output.setText(s.toString());
            output.post(this::scrollOutputToBottom);
        });
    }

    private void sendStdInput(String text) {
        if (proc == null || text.isEmpty()) return;
        synchronized (outLines) {
            outPartial.append(text);
            String echo = outPartial.toString();
            outPartial.setLength(0);
            outLines.addLast(echo);
            while (outLines.size() > MAX_OUTPUT_LINES) outLines.removeFirst();
        }
        flushOutputUi();
        try {
            OutputStream os = proc.getOutputStream();
            os.write((text + "\n").getBytes(StandardCharsets.UTF_8));
            os.flush();
        } catch (IOException ignored) {
        }
    }

    private void scrollOutputToBottom() {
        if (output.getLayout() == null) return;
        int target = output.getLayout().getHeight() + output.getCompoundPaddingTop()
                + output.getCompoundPaddingBottom() - output.getHeight();
        output.scrollTo(0, Math.max(0, target));
    }

    private void copyOutput() {
        ClipboardManager cm = (ClipboardManager) getSystemService(CLIPBOARD_SERVICE);
        cm.setPrimaryClip(ClipData.newPlainText("shifra-output", output.getText().toString()));
        Toast.makeText(this, R.string.copy_done, Toast.LENGTH_SHORT).show();
    }
}