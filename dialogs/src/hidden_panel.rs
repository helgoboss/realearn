use crate::base::*;

pub fn create(context: ScopedContext, ids: &mut IdGenerator) -> Dialog {
    Dialog {
        id: ids.named_id("ID_HIDDEN_PANEL"),
        caption: "Hidden panel",
        ..context.default_dialog()
    }
}
