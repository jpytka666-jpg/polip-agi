# DARKSTAR HMI face

Drop-in UI payload for the ES3C35P / ESP32-S3 front-panel project.

This branch deliberately contains only the UI layer. It does not assume LCD, touch, audio or USB pin mappings. Codex should first finish hardware verification against the exact ES3C35P revision, then integrate this UI into the verified board project.

## Visual contract

- portrait 320x480
- black background
- phosphor green = normal/online
- amber = standby/warning
- red = alarm only
- no rounded mobile cards, gradients, emoji or generic dashboard chrome
- industrial / military / late-80s machine terminal aesthetic
- four touch softkeys: SYS / NET / GPU / CORE
- host-loss state must dominate the center of the display

## Integration

Copy `darkstar_face.c` and `darkstar_face.h` into the ESP-IDF/LVGL firmware project and call:

```c
darkstar_face_init(lv_scr_act());
darkstar_face_set_status(&state);
```

For LVGL 9 replace `lv_scr_act()` with `lv_screen_active()` at the call site if required.

The code intentionally uses only common LVGL object/style APIs and leaves fonts overrideable.

Do not flash merely because this UI compiles. Keep the already VERIFIED factory backup as the recovery point.
