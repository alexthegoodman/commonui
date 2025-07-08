pub struct Signal<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Signal<T> {
    pub fn new(_value: T) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}