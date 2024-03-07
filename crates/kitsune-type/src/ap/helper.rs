use super::{Object, PUBLIC_IDENTIFIER};

pub trait CcTo {
    fn cc(&self) -> &[String];
    fn to(&self) -> &[String];
}

impl<T> CcTo for Box<T>
where
    T: CcTo,
{
    fn cc(&self) -> &[String] {
        (**self).cc()
    }

    fn to(&self) -> &[String] {
        (**self).to()
    }
}

impl CcTo for Object {
    fn cc(&self) -> &[String] {
        &self.cc
    }

    fn to(&self) -> &[String] {
        &self.to
    }
}

pub trait Privacy {
    fn is_public(&self) -> bool;
    fn is_unlisted(&self) -> bool;

    fn is_private(&self) -> bool {
        !self.is_public() && !self.is_unlisted()
    }
}

impl<T> Privacy for Box<T>
where
    T: Privacy,
{
    fn is_public(&self) -> bool {
        (**self).is_public()
    }

    fn is_unlisted(&self) -> bool {
        (**self).is_unlisted()
    }

    fn is_private(&self) -> bool {
        (**self).is_private()
    }
}

impl Privacy for Object {
    fn is_public(&self) -> bool {
        self.to.iter().any(|url| url == PUBLIC_IDENTIFIER)
    }

    fn is_unlisted(&self) -> bool {
        !self.is_public() && self.cc.iter().any(|url| url == PUBLIC_IDENTIFIER)
    }
}
