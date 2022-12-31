use pest_derive::Parser;

/// Pest-based parser
#[derive(Parser)]
#[grammar = "../grammar/post.pest"]
pub struct PostParser;
