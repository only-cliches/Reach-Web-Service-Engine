use std::fmt::Debug;

pub trait LogToOk<T> {
    fn log_to_ok(self) -> Option<T>;
}

impl<T, E> LogToOk<T> for Result<T, E>
where
    E: Debug,
{
    fn log_to_ok(self) -> Option<T> {
        match self {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("{:?}", e);
                None
            }
        }
    }
}
