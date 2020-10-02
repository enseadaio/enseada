pub type FnPredicate<T> = Box<dyn FnOnce(&T) -> bool>;

pub fn not<T, P: 'static + FnOnce(&T) -> bool>(predicate: P) -> FnPredicate<T> {
    Box::new(|t| !predicate(t))
}

pub fn eq<T: 'static + PartialEq>(other: T) -> FnPredicate<T> {
    Box::new(move |t| t.eq(&other))
}

pub fn ne<T: 'static + PartialEq>(other: T) -> FnPredicate<T> {
    not(eq(other))
}
