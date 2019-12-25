pub(crate) fn empty_or_none<T>(input: &[T]) -> Option<&[T]> {
    if input.is_empty() {
        None
    } else {
        Some(input)
    }
}

pub(crate) fn copy_on_push_vec<T>(input: &[T], el: T) -> Vec<T>
where
    T: Clone,
{
    let mut new_vec = Vec::with_capacity(input.len() + 1);
    new_vec.extend_from_slice(input);
    new_vec.push(el);
    new_vec
}
