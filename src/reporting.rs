#[derive(Debug)]
pub enum UpdateError {
    LogicalBlockWriteError(LogicalBlockError),
    MissingLogicalBlockError(LogicalBlockError),
}

#[derive(Debug)]
pub struct LogicalBlockError {
    pub logical_block_id: String,
    pub description: String,
}
