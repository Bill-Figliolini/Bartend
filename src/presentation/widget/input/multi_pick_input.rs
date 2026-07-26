use std::collections::HashSet;

// could I turn the structure  {id, body} into a trait, to make things smoother here?
struct MultipickInput<T> {
    choices: Vec<T>,
    selected: HashSet<T>,
}
