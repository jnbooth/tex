pub fn drain_filter<F, T>(vec: &mut Vec<T>, filter: F) -> Vec<T> where F: Fn(&T) -> bool {
    let mut drained = Vec::new();
    let mut i = 0;
    while i != vec.len() {
        if filter(&vec[i]) {
            drained.push(vec.remove(i));
        } else {
            i += 1;
        }
    }
    drained
}

pub fn pop_filter<F, T>(vec: &mut Vec<T>, filter: F) -> Option<T> where F: Fn(&T) -> bool {
    for i in 0..vec.len() {
        if filter(&vec[i]) {
            return Some(vec.remove(i))
        }
    }
    None
}
