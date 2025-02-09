pub fn ignore<T>(_: T) {}

pub trait TToOption<T> {
    fn to_option(self) -> Option<T>;
}

impl<T, E> TToOption<T> for Result<T, E> {
    fn to_option(self) -> Option<T> {
        if let Ok(value) = self {
            Some(value)
        } else {
            None
        }
    }
}
