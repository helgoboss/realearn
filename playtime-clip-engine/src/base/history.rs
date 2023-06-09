use crate::ClipEngineResult;
use playtime_api::persistence as api;

/// Data structure holding the undo history.
#[derive(Debug)]
pub struct History {
    undo_stack: Vec<State>,
    redo_stack: Vec<State>,
}

impl History {
    pub fn new(initial_matrix: api::Matrix) -> Self {
        Self {
            undo_stack: vec![State::new("Initial".to_string(), initial_matrix)],
            redo_stack: vec![],
        }
    }

    /// Returns the label of the next undoable action if there is one.
    pub fn next_undo_label(&self) -> Option<&str> {
        if !self.can_undo() {
            return None;
        }
        let state = self.undo_stack.last()?;
        Some(&state.label)
    }

    /// Returns the label of the next redoable action if there is one.
    pub fn next_redo_label(&self) -> Option<&str> {
        let state = self.redo_stack.last()?;
        Some(&state.label)
    }

    /// Returns if undo is possible.
    pub fn can_undo(&self) -> bool {
        self.undo_stack.len() > 1
    }

    /// Returns if redo is possible.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Adds the given history entry if the matrix is different from the one in the previous
    /// undo point.
    pub fn add(&mut self, label: String, new_matrix: api::Matrix) {
        if let Some(prev_state) = self.undo_stack.last() {
            if new_matrix == prev_state.matrix {
                return;
            }
        };
        self.redo_stack.clear();
        let new_state = State::new(label, new_matrix);
        self.undo_stack.push(new_state);
    }

    /// Marks the last action as undone and returns the matrix state to be loaded.
    pub fn undo(&mut self) -> ClipEngineResult<&api::Matrix> {
        if self.undo_stack.len() <= 1 {
            return Err("nothing to undo");
        }
        let state = self.undo_stack.pop().unwrap();
        self.redo_stack.push(state);
        Ok(&self.undo_stack.last().unwrap().matrix)
    }

    /// Marks the last undone action as redone and returns the matrix state to be loaded.
    pub fn redo(&mut self) -> ClipEngineResult<&api::Matrix> {
        let state = self.redo_stack.pop().ok_or("nothing to redo")?;
        self.undo_stack.push(state);
        Ok(&self.undo_stack.last().unwrap().matrix)
    }
}

// TODO-medium Make use of label
#[allow(dead_code)]
#[derive(Debug)]
struct State {
    label: String,
    matrix: api::Matrix,
}

impl State {
    fn new(label: String, matrix: api::Matrix) -> Self {
        Self { label, matrix }
    }
}
