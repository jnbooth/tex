use crate::IO;
use super::NO_RESULTS;

const NOT_FOUND: &str = "That isn't one of my choices.";
const CHARACTER_LIMIT: usize = 429;

pub struct Choices(Vec<Box<Fn() -> IO<String>>>);

impl Choices {
    pub fn new() -> Self {
        Choices(Vec::new())
    }

    pub fn add<F>(&mut self, callback: F) where F: 'static + Fn() -> IO<String> {
        self.0.push(Box::new(callback));
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn run_choice(&mut self, i: usize) -> IO<String> {
        match self.0.get(i) {
            Some(choice) => (choice)(),
            None => Ok(NOT_FOUND.to_owned())
        }
    }
}

pub fn suggest(suggestions: &[String]) -> String {
    if suggestions.is_empty() {
        NO_RESULTS.to_owned()
    } else {
        let mut s = "Did you mean:".to_owned();
        let mut i = 0;
        for suggest in suggestions {
            i += 1;
            if s.len() + suggest.len() + 7 > CHARACTER_LIMIT {
                return s.to_owned()
            }
            if i > 1 {
                s.push_str(",");
            }
            s.push_str(" (");
            s.push_str(&i.to_string());
            s.push_str(") ");
            s.push_str(suggest);
        }
        s.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_run_choice() {
        let mut x = Choices::new();
        x.add(|| Ok("success!".to_owned()));
        assert_eq!(x.run_choice(0).unwrap(), "success!".to_owned());
    }

    #[test]
    fn test_empty() {
        assert_eq!(Choices::new().run_choice(0).unwrap(), NOT_FOUND.to_owned());
    }

    #[test]
    fn test_outsize() {
        let mut x = Choices::new();
        x.add(|| panic!("Wrong choice picked!"));
        assert_eq!(x.run_choice(1).unwrap(), NOT_FOUND.to_owned());
    }

    #[test]
    fn test_clear() {
        let mut x = Choices::new();
        x.add(|| panic!("Wrong choice picked!"));
        x.clear();
        assert_eq!(x.run_choice(0).unwrap(), NOT_FOUND.to_owned());
    }
}
