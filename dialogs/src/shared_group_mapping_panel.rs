use crate::base::*;
use crate::ext::*;

pub fn create(mut context: ScopedContext) -> Dialog {
    use Style::*;
    let line_2_y = 20;
    let controls = vec![
        // Name
        ltext("Name", context.id(), context.rect(5, 6, 20, 9)) + NOT_WS_GROUP,
        edittext(
            context.named_id("ID_MAPPING_NAME_EDIT_CONTROL"),
            context.rect(33, 3, 131, 14),
        ) + ES_MULTILINE
            + ES_AUTOHSCROLL,
        // Tags
        ltext("Tags", context.id(), context.rect(172, 6, 18, 9)) + NOT_WS_GROUP,
        edittext(
            context.named_id("ID_MAPPING_TAGS_EDIT_CONTROL"),
            context.rect(194, 3, 131, 14),
        ) + ES_MULTILINE
            + ES_AUTOHSCROLL,
        // Control/feedback checkboxes
        checkbox(
            "=> Control",
            context.named_id("ID_MAPPING_CONTROL_ENABLED_CHECK_BOX"),
            context.rect(330, 6, 50, 8),
        ) + WS_TABSTOP,
        checkbox(
            "<= Feedback",
            context.named_id("ID_MAPPING_FEEDBACK_ENABLED_CHECK_BOX"),
            context.rect(381, 6, 56, 8),
        ) + WS_TABSTOP,
        // Conditional activation
        ltext(
            "Active",
            context.named_id("ID_MAPPING_ACTIVATION_LABEL"),
            context.rect(5, line_2_y + 2, 21, 9),
        ) + NOT_WS_GROUP,
        dropdown(
            context.named_id("ID_MAPPING_ACTIVATION_TYPE_COMBO_BOX"),
            context.rect(33, line_2_y, 102, 15),
        ) + WS_TABSTOP,
        // Conditional activation criteria 1
        ltext(
            "Modifier 1",
            context.named_id("ID_MAPPING_ACTIVATION_SETTING_1_LABEL_TEXT"),
            context.rect(143, line_2_y + 2, 33, 9),
        ) + NOT_WS_GROUP,
        dropdown(
            context.named_id("ID_MAPPING_ACTIVATION_SETTING_1_COMBO_BOX"),
            context.rect(182, line_2_y, 90, 15),
        ) + WS_VSCROLL
            + WS_TABSTOP,
        checkbox(
            "",
            context.named_id("ID_MAPPING_ACTIVATION_SETTING_1_CHECK_BOX"),
            context.rect(276, line_2_y + 2, 11, 8),
        ) + WS_TABSTOP,
        // Conditional activation criteria 2
        ltext(
            "Modifier 2",
            context.named_id("ID_MAPPING_ACTIVATION_SETTING_2_LABEL_TEXT"),
            context.rect(292, line_2_y + 2, 34, 9),
        ) + NOT_WS_GROUP,
        dropdown(
            context.named_id("ID_MAPPING_ACTIVATION_SETTING_2_COMBO_BOX"),
            context.rect(330, line_2_y, 90, 15),
        ) + WS_VSCROLL
            + WS_TABSTOP,
        checkbox(
            "",
            context.named_id("ID_MAPPING_ACTIVATION_SETTING_2_CHECK_BOX"),
            context.rect(424, line_2_y + 2, 11, 8),
        ) + WS_TABSTOP,
        ltext(
            "EEL (e.g. y = p1 > 0)",
            context.named_id("ID_MAPPING_ACTIVATION_EEL_LABEL_TEXT"),
            context.rect(143, line_2_y + 2, 70, 9),
        ) + NOT_WS_GROUP,
        edittext(
            context.named_id("ID_MAPPING_ACTIVATION_EDIT_CONTROL"),
            context.rect(213, line_2_y, 220, 14),
        ) + ES_MULTILINE
            + ES_AUTOHSCROLL,
    ];
    Dialog {
        id: context.named_id("ID_SHARED_GROUP_MAPPING_PANEL"),
        kind: DialogKind::DIALOGEX,
        rect: context.rect(0, 0, 440, 37),
        styles: Styles(vec![
            DS_SETFONT, DS_CONTROL, DS_CENTER, WS_CHILD, WS_VISIBLE, WS_SYSMENU,
        ]),
        controls,
        ..context.default_dialog()
    }
}
