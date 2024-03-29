use crate::base::*;
use crate::constants::{MAPPING_ROWS_PANEL_HEIGHT, MAPPING_ROWS_PANEL_WIDTH};

pub fn create(context: ScopedContext, ids: &mut IdGenerator) -> Dialog {
    use Style::*;
    let controls = [
        pushbutton(
            "Display mappings in all groups",
            ids.named_id("ID_DISPLAY_ALL_GROUPS_BUTTON"),
            context.rect(157, 137, 156, 14),
        ),
        ctext(
            "There are no mappings in this compartment.",
            ids.named_id("ID_GROUP_IS_EMPTY_TEXT"),
            context.rect(149, 121, 173, 9),
        ) + NOT_WS_GROUP,
    ];
    Dialog {
        id: ids.named_id("ID_MAPPING_ROWS_PANEL"),
        kind: DialogKind::DIALOGEX,
        rect: context.rect(0, 0, MAPPING_ROWS_PANEL_WIDTH, MAPPING_ROWS_PANEL_HEIGHT),
        styles: Styles(vec![DS_SETFONT, DS_CONTROL, WS_CHILD, WS_VISIBLE]),
        controls: controls.into_iter().collect(),
        ..context.default_dialog()
    }
}
