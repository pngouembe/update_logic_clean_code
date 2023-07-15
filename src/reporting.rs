#[derive(Debug, PartialEq)]
pub enum UpdateError {
    LogicalBlockWrite(LogicalBlockError),
    LogicalBlockRead(LogicalBlockError),
    MissingLogicalBlock(LogicalBlockError),
    LogicalBlockSize(LogicalBlockError),
    VerificationError(LogicalBlockError),
}

#[derive(Debug, PartialEq)]
pub struct LogicalBlockError {
    pub logical_block_id: String,
    pub description: String,
}
