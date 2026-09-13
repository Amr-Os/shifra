package com.shifra.language;

import android.app.Activity;
import android.content.SharedPreferences;
import android.os.Bundle;
import android.view.View;
import android.view.ViewGroup;
import android.view.WindowManager;
import android.widget.AdapterView;
import android.widget.ArrayAdapter;
import android.widget.Button;
import android.widget.EditText;
import android.widget.SeekBar;
import android.widget.Spinner;
import android.widget.Switch;
import android.widget.TextView;

import java.util.Arrays;

public class SettingsActivity extends Activity {

    private static final String PREFS = "shifra";

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN);
        setContentView(R.layout.settings);
        MainActivity.enableImmersive(this);

        final SharedPreferences p = getSharedPreferences(PREFS, MODE_PRIVATE);

        final EditText snips = findViewById(R.id.snips_edit);
        snips.setText(MainActivity.snippetFieldText(this));
        Button snipsSave = findViewById(R.id.snips_save);
        snipsSave.setOnClickListener(v ->
                MainActivity.saveSnippetFieldText(this, snips.getText().toString()));

        final Spinner fontPicker = findViewById(R.id.font_spinner);
        final String[] families = {"monospace", "sans", "serif"};
        final String[] labels = {
                getString(R.string.font_mono),
                getString(R.string.font_sans),
                getString(R.string.font_serif)
        };
        ArrayAdapter<String> adapter = new ArrayAdapter<String>(this, android.R.layout.simple_spinner_item, labels) {
            @Override
            public View getView(int position, View convertView, ViewGroup parent) {
                TextView tv = (TextView) super.getView(position, convertView, parent);
                tv.setTextColor(getColor(R.color.text));
                return tv;
            }

            @Override
            public View getDropDownView(int position, View convertView, ViewGroup parent) {
                TextView tv = (TextView) super.getDropDownView(position, convertView, parent);
                tv.setTextColor(getColor(R.color.text));
                return tv;
            }
        };
        fontPicker.setAdapter(adapter);
        int cur = Arrays.asList(families).indexOf(p.getString("font_family", "monospace"));
        fontPicker.setSelection(Math.max(0, cur));
        fontPicker.setOnItemSelectedListener(new AdapterView.OnItemSelectedListener() {
            @Override
            public void onItemSelected(AdapterView<?> parent, View view, int position, long id) {
                p.edit().putString("font_family", families[position]).apply();
            }

            @Override
            public void onNothingSelected(AdapterView<?> parent) {
            }
        });

        final TextView sizeLabel = findViewById(R.id.size_label);
        final SeekBar size = findViewById(R.id.size_seek);
        int sp = Math.round(p.getFloat("font_size", 15f));
        size.setProgress(Math.max(0, Math.min(14, sp - 12)));
        sizeLabel.setText(getString(R.string.settings_font_size) + ": " + sp);
        size.setOnSeekBarChangeListener(new SeekBar.OnSeekBarChangeListener() {
            @Override
            public void onProgressChanged(SeekBar sb, int progress, boolean fromUser) {
                int val = progress + 12;
                sizeLabel.setText(getString(R.string.settings_font_size) + ": " + val);
                p.edit().putFloat("font_size", val).apply();
            }

            @Override
            public void onStartTrackingTouch(SeekBar sb) {
            }

            @Override
            public void onStopTrackingTouch(SeekBar sb) {
            }
        });

        final Switch wrap = findViewById(R.id.wrap_switch);
        wrap.setChecked(p.getBoolean("wrap", false));
        wrap.setOnCheckedChangeListener((btn, isChecked) ->
                p.edit().putBoolean("wrap", isChecked).apply());
    }
}