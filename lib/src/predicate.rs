pub fn not<T, P: 'static + FnOnce(&T) -> bool>(predicate: P) -> Box<dyn FnOnce(&T) -> bool> {
    Box::new(|t| !predicate(t))
}
