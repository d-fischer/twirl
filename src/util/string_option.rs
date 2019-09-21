pub trait StringOption {
    fn get(self) -> Option<String>;
}

impl StringOption for &'_ str {
    fn get(self) -> Option<String> {
        Some(self.into())
    }
}

impl StringOption for String {
    fn get(self) -> Option<String> {
        Some(self)
    }
}

impl StringOption for () {
    fn get(self) -> Option<String> {
        None
    }
}

impl StringOption for Option<String> {
    fn get(self) -> Option<String> {
        self
    }
}
