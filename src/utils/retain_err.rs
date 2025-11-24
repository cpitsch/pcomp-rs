use std::error::Error;

/// [`Vec::retain`], but the filter function can error. In this case, the error is propagated
/// upwards and the vec remains untouched.
pub fn retain_err<T, F, E>(vec: &mut Vec<T>, f: F) -> Result<(), E>
where
    F: FnMut(&T) -> Result<bool, E>,
    E: Error,
{
    let mut retain_indices = vec
        .iter()
        .map(f)
        .collect::<Result<Vec<bool>, E>>()?
        .into_iter();

    vec.retain(|_| retain_indices.next().unwrap());

    Ok(())
}

/// [`Vec::retain_mut`], but the filter function can error. In this case, the error is propagated
/// upwards and the vec remains untouched (with the exception of any changes made by applying `f`.
pub fn retain_mut_err<T, F, E>(vec: &mut Vec<T>, f: F) -> Result<(), E>
where
    F: FnMut(&mut T) -> Result<bool, E>,
    E: Error,
{
    let mut retain_indices = vec
        .iter_mut()
        .map(f)
        .collect::<Result<Vec<bool>, E>>()?
        .into_iter();

    vec.retain(|_| retain_indices.next().unwrap());

    Ok(())
}
