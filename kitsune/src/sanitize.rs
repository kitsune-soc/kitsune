use kitsune_type::ap::object::{Actor, Note};

pub trait CleanHtmlExt {
    fn clean_html(&mut self);
}

impl CleanHtmlExt for Actor {
    fn clean_html(&mut self) {
        if let Some(ref mut name) = self.name {
            name.clean_html();
        }

        if let Some(ref mut subject) = self.subject {
            subject.clean_html();
        }
    }
}

impl CleanHtmlExt for Note {
    fn clean_html(&mut self) {
        if let Some(ref mut summary) = self.summary {
            summary.clean_html();
        }

        self.content.clean_html();
    }
}

impl CleanHtmlExt for String {
    fn clean_html(&mut self) {
        *self = ammonia::clean(self);
    }
}
