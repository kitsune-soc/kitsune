pub trait RdfNode {
    fn id(&self) -> Option<&str>;
}
