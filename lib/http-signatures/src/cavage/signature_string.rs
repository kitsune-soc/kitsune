use http::Method;

pub const REQUIRED_GET_HEADERS: &[&str] = &["host", "date"];
pub const REQUIRED_POST_HEADERS: &[&str] = &["host", "date", "content-type", "digest"];

#[inline]
fn is_subset<I>(left: &[I], right: &[I]) -> bool
where
    I: PartialEq,
{
    if left.len() <= right.len() {
        left.iter().all(|item| right.contains(item))
    } else {
        false
    }
}

#[inline]
pub fn construct<B>(request: &http::Request<B>, header_names: &[&str]) -> Result<(), ()> {
    let fulfills_min_requirements = match *request.method() {
        Method::GET => is_subset(REQUIRED_GET_HEADERS, header_names),
        Method::POST => is_subset(REQUIRED_POST_HEADERS, header_names),
        _ => todo!("how should we handle this?"),
    };

    if !fulfills_min_requirements {
        return Err(());
    }

    todo!();
}

#[cfg(test)]
mod test {
    use super::is_subset;
    use proptest::{prop_assert_eq, proptest};
    use std::collections::HashSet;

    proptest! {
        #[test]
        fn subset_behaves_equal(left: HashSet<String>, right: HashSet<String>) {
            let vec_left = left.iter().collect::<Vec<_>>();
            let vec_right = right.iter().collect::<Vec<_>>();

            let slice_subset = is_subset(&vec_left, &vec_right);
            let set_subset = left.is_subset(&right);

            prop_assert_eq!(slice_subset, set_subset);
        }
    }
}
