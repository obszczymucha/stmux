pub(crate) trait Session {
    fn find(&self);
    fn select(&self, name: &str);
}

pub(crate) struct SessionImpl;

impl Session for SessionImpl {
    fn find(&self) {
        unimplemented!()
    }

    fn select(&self, name: &str) {
        eprintln!("Selecting session: {}", name);
    }
}
