use super::NO_RESULTS;
use super::super::IO;

const CHARACTER_LIMIT: usize = 429;

pub struct Choices {
    choices: Vec<Box<Fn() -> IO<String>>>
}
impl Choices {
    pub fn new() -> Choices {
        Choices { choices: Vec::new() }
    }

    pub fn add<F>(&mut self, callback: F) where F: 'static + Fn() -> IO<String> {
        self.choices.push(Box::new(callback));
    }

    pub fn clear(&mut self) {
        self.choices.clear();
    }

    pub fn run_choice(&mut self, i: usize) -> IO<String> {
        match self.choices.get(i - 1) {
            Some(choice) => (choice)(),
            None => Ok("That isn't one of my choices.".to_string())
        }
    }
}

pub fn suggest(suggestions: &Vec<String>) -> String {
    if suggestions.is_empty() {
        NO_RESULTS.to_owned()
    } else {
        let mut s = "Did you mean:".to_owned();
        let mut i = 0;
        for suggest in suggestions {
            i = i + 1;
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
