use bubble_bath::BubbleBath;
use kitsune_type::ap::{actor::Actor, Object};
use once_cell::sync::Lazy;

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

impl CleanHtmlExt for Object {
    fn clean_html(&mut self) {
        if let Some(ref mut summary) = self.summary {
            summary.clean_html();
        }

        self.content.clean_html();
    }
}

impl CleanHtmlExt for String {
    fn clean_html(&mut self) {
        static BUBBLE_BATH: Lazy<BubbleBath<'static>> = Lazy::new(|| BubbleBath {
            preserve_escaped: true,
            ..BubbleBath::default()
        });

        *self = BUBBLE_BATH.clean(self).unwrap();
    }
}
