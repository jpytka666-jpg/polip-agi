#include "darkstar_face.h"
#include <stdio.h>

#ifndef DARKSTAR_FONT_HEADER
#define DARKSTAR_FONT_HEADER LV_FONT_DEFAULT
#endif
#ifndef DARKSTAR_FONT_BODY
#define DARKSTAR_FONT_BODY LV_FONT_DEFAULT
#endif

#define C_BG        lv_color_hex(0x050806)
#define C_PANEL     lv_color_hex(0x0A100C)
#define C_GREEN     lv_color_hex(0x7CFF8A)
#define C_GREEN_DIM lv_color_hex(0x315C39)
#define C_AMBER     lv_color_hex(0xF2B84B)
#define C_RED       lv_color_hex(0xFF4D3F)
#define C_TEXT      lv_color_hex(0xD9E6DA)
#define C_DIM       lv_color_hex(0x6C7A70)

typedef struct {
    lv_obj_t *root;
    lv_obj_t *title;
    lv_obj_t *node;
    lv_obj_t *rows[5];
    lv_obj_t *names[5];
    lv_obj_t *values[5];
    lv_obj_t *alarm_box;
    lv_obj_t *alarm_title;
    lv_obj_t *alarm_time;
} face_ctx_t;

static face_ctx_t g;

static lv_color_t status_color(darkstar_status_t s) {
    switch (s) {
        case DARKSTAR_STATUS_ONLINE:  return C_GREEN;
        case DARKSTAR_STATUS_STANDBY: return C_AMBER;
        case DARKSTAR_STATUS_WARNING: return C_AMBER;
        case DARKSTAR_STATUS_ALARM:   return C_RED;
        case DARKSTAR_STATUS_OFFLINE:
        default:                      return C_DIM;
    }
}

static void style_label(lv_obj_t *o, lv_color_t color) {
    lv_obj_set_style_text_color(o, color, 0);
    lv_obj_set_style_text_font(o, DARKSTAR_FONT_BODY, 0);
    lv_obj_set_style_bg_opa(o, LV_OPA_TRANSP, 0);
}

static lv_obj_t *make_label(lv_obj_t *parent, const char *text, int x, int y) {
    lv_obj_t *l = lv_label_create(parent);
    lv_label_set_text(l, text);
    lv_obj_set_pos(l, x, y);
    style_label(l, C_TEXT);
    return l;
}

static void make_softkey(lv_obj_t *parent, const char *text, int x) {
    lv_obj_t *box = lv_obj_create(parent);
    lv_obj_set_pos(box, x, 430);
    lv_obj_set_size(box, 72, 38);
    lv_obj_set_style_radius(box, 0, 0);
    lv_obj_set_style_bg_color(box, C_PANEL, 0);
    lv_obj_set_style_bg_opa(box, LV_OPA_COVER, 0);
    lv_obj_set_style_border_color(box, C_GREEN_DIM, 0);
    lv_obj_set_style_border_width(box, 1, 0);
    lv_obj_set_style_pad_all(box, 0, 0);

    lv_obj_t *l = lv_label_create(box);
    lv_label_set_text(l, text);
    style_label(l, C_GREEN);
    lv_obj_center(l);
}

void darkstar_face_init(lv_obj_t *root) {
    g.root = root;

    lv_obj_set_style_bg_color(root, C_BG, 0);
    lv_obj_set_style_bg_opa(root, LV_OPA_COVER, 0);
    lv_obj_set_style_pad_all(root, 0, 0);
    lv_obj_clear_flag(root, LV_OBJ_FLAG_SCROLLABLE);

    g.title = make_label(root, "DARKSTAR", 14, 14);
    lv_obj_set_style_text_color(g.title, C_GREEN, 0);
    lv_obj_set_style_text_font(g.title, DARKSTAR_FONT_HEADER, 0);

    g.node = make_label(root, "R730 / NODE 01", 15, 43);
    lv_obj_set_style_text_color(g.node, C_DIM, 0);

    lv_obj_t *rule = lv_obj_create(root);
    lv_obj_set_pos(rule, 14, 70);
    lv_obj_set_size(rule, 292, 2);
    lv_obj_set_style_border_width(rule, 0, 0);
    lv_obj_set_style_radius(rule, 0, 0);
    lv_obj_set_style_bg_color(rule, C_GREEN_DIM, 0);
    lv_obj_set_style_bg_opa(rule, LV_OPA_COVER, 0);
    lv_obj_set_style_pad_all(rule, 0, 0);

    const char *names[5] = {"HOST", "AIONS", "NET", "GPU", "THERMAL"};
    const int ys[5] = {104, 154, 204, 254, 304};

    for (int i = 0; i < 5; ++i) {
        g.rows[i] = lv_obj_create(root);
        lv_obj_set_pos(g.rows[i], 14, ys[i] - 8);
        lv_obj_set_size(g.rows[i], 292, 40);
        lv_obj_set_style_radius(g.rows[i], 0, 0);
        lv_obj_set_style_bg_color(g.rows[i], C_PANEL, 0);
        lv_obj_set_style_bg_opa(g.rows[i], LV_OPA_COVER, 0);
        lv_obj_set_style_border_color(g.rows[i], C_GREEN_DIM, 0);
        lv_obj_set_style_border_width(g.rows[i], 1, 0);
        lv_obj_set_style_pad_all(g.rows[i], 0, 0);

        g.names[i] = make_label(g.rows[i], names[i], 10, 9);
        lv_obj_set_style_text_color(g.names[i], C_DIM, 0);

        g.values[i] = make_label(g.rows[i], "--", 174, 9);
        lv_obj_set_width(g.values[i], 104);
        lv_obj_set_style_text_align(g.values[i], LV_TEXT_ALIGN_RIGHT, 0);
    }

    make_softkey(root, "SYS", 14);
    make_softkey(root, "NET", 92);
    make_softkey(root, "GPU", 170);
    make_softkey(root, "CORE", 248);

    g.alarm_box = lv_obj_create(root);
    lv_obj_set_pos(g.alarm_box, 14, 92);
    lv_obj_set_size(g.alarm_box, 292, 282);
    lv_obj_set_style_radius(g.alarm_box, 0, 0);
    lv_obj_set_style_bg_color(g.alarm_box, C_BG, 0);
    lv_obj_set_style_bg_opa(g.alarm_box, LV_OPA_COVER, 0);
    lv_obj_set_style_border_color(g.alarm_box, C_RED, 0);
    lv_obj_set_style_border_width(g.alarm_box, 2, 0);
    lv_obj_set_style_pad_all(g.alarm_box, 0, 0);

    g.alarm_title = make_label(g.alarm_box, "HOST LINK LOST", 38, 92);
    lv_obj_set_style_text_color(g.alarm_title, C_RED, 0);
    lv_obj_set_style_text_font(g.alarm_title, DARKSTAR_FONT_HEADER, 0);

    g.alarm_time = make_label(g.alarm_box, "LAST CONTACT: 00:00:00", 32, 142);
    lv_obj_set_style_text_color(g.alarm_time, C_AMBER, 0);

    lv_obj_add_flag(g.alarm_box, LV_OBJ_FLAG_HIDDEN);
}

void darkstar_face_set_status(const darkstar_face_state_t *s) {
    if (!s || !g.root) return;

    if (s->node_name) lv_label_set_text(g.node, s->node_name);

    const darkstar_status_t st[5] = {s->host, s->aions, s->net, s->gpu, s->thermal};
    const char *txt[5] = {
        s->host_text ? s->host_text : "--",
        s->aions_text ? s->aions_text : "--",
        s->net_text ? s->net_text : "--",
        s->gpu_text ? s->gpu_text : "--",
        s->thermal_text ? s->thermal_text : "--"
    };

    for (int i = 0; i < 5; ++i) {
        lv_label_set_text(g.values[i], txt[i]);
        lv_obj_set_style_text_color(g.values[i], status_color(st[i]), 0);
    }

    if (s->host_lost) {
        char buf[48];
        uint32_t t = s->seconds_since_contact;
        snprintf(buf, sizeof(buf), "LAST CONTACT: %02lu:%02lu:%02lu",
                 (unsigned long)(t / 3600),
                 (unsigned long)((t / 60) % 60),
                 (unsigned long)(t % 60));
        lv_label_set_text(g.alarm_time, buf);
        lv_obj_clear_flag(g.alarm_box, LV_OBJ_FLAG_HIDDEN);
    } else {
        lv_obj_add_flag(g.alarm_box, LV_OBJ_FLAG_HIDDEN);
    }
}
