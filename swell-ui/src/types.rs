use std::rc::Rc;

pub type SharedView<V> = Rc<V>;
pub type WeakView<V> = std::rc::Weak<V>;
