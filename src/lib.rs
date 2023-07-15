mod reporting;

mod multi_threaded_update;
pub use crate::multi_threaded_update::update_sequence::multi_threaded_update;

mod sequential_update;
pub use crate::sequential_update::update_sequence::sequencial_update;
