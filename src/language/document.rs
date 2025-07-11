/// A general document version, that always increments overall like version of a software
/// 
/// This is a simple way to know if the content of a document might have changed.
/// Different versions doesn't mean that the content is actually changed, it could stil be the same.
/// 
/// Note when we say document, if the document is open in client, it is the content in editor not filesystem.
/// But when file is closed, it is the content on the filesystem.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DocumentVersion{
    /// This version will be incremented every time file is opened or closed in a client
    /// If the client closed a file, then the minor version is reset to 0
    pub major: i32,
    /// Minor version
    /// 
    /// When file is open in client, then this version is the version of the document content of the client, as reported by the client, not on the file system
    /// 
    /// When file is not open in client, then this version is the version of the document content on the file system.
    /// When change is detected on filesystem when file is closed in client, we should increment this version.
    /// But this does not mean we will always detect changes on the filesystem.
    pub minor: i32,
}
