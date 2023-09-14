//

// All undoable actions must implement this trait
pub trait Action {
    fn execute(&self);
    fn undo(&self);
}