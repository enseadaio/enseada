/// Backports of unstable Rust features

pub mod option {
    /// Zips `this` with another `Option`.
    ///
    /// If `this` is `Some(s)` and `other` is `Some(o)`, this method returns `Some((s, o))`.
    /// Otherwise, `None` is returned.
    ///
    /// Backport of unstable `Option::zip`
    ///
    /// # Examples
    ///
    /// ```
    /// use enseada::backports::option::zip;
    ///
    /// let x = Some(1);
    /// let y = Some("hi");
    /// let z = None::<u8>;
    ///
    /// assert_eq!(zip(x, y), Some((1, "hi")));
    /// assert_eq!(zip(x, z), None);
    /// ```
    pub fn zip<T, U>(this: Option<T>, other: Option<U>) -> Option<(T, U)> {
        zip_with(this, other, |a, b| (a, b))
    }

    /// Zips `this` and another `Option` with function `f`.
    ///
    /// If `this` is `Some(s)` and `other` is `Some(o)`, this method returns `Some(f(s, o))`.
    /// Otherwise, `None` is returned.
    ///
    /// Backport of unstable `Option::zip_with`
    ///
    /// # Examples
    ///
    /// ```
    /// use enseada::backports::option::zip_with;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Point {
    ///     x: f64,
    ///     y: f64,
    /// }
    ///
    /// impl Point {
    ///     fn new(x: f64, y: f64) -> Self {
    ///         Self { x, y }
    ///     }
    /// }
    ///
    /// let x = Some(17.5);
    /// let y = Some(42.7);
    ///
    /// assert_eq!(zip_with(x, y, Point::new), Some(Point { x: 17.5, y: 42.7 }));
    /// assert_eq!(zip_with(x, None, Point::new), None);
    /// ```
    pub fn zip_with<T, U, F, R>(this: Option<T>, other: Option<U>, f: F) -> Option<R>
    where
        F: FnOnce(T, U) -> R,
    {
        Some(f(this?, other?))
    }
}
