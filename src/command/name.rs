use rand::distributions::{Distribution, WeightedError, WeightedIndex};
use rand::Rng;

use super::*;
use crate::IO;
use crate::util::Gender;

use crate::db::establish_connection;

#[derive(Debug, Clone)]
struct NameList {
    names: Vec<String>,
    dist:  WeightedIndex<i32>
}
impl NameList {
    pub fn build(names: &[db::Name]) -> Result<NameList, WeightedError> {
        Ok(Self { 
            names: names.into_iter().map(|x| x.name.to_owned()).collect(),
            dist:  WeightedIndex::new(names.into_iter().map(|x| x.frequency))?
        })
    }
    pub fn choose<T: Rng>(&self, rng: &mut T) -> String {
        self.names[self.dist.sample(rng)].to_owned()
    }
}

#[derive(Debug, Clone)]
pub struct Name {
    male: NameList,
    female: NameList,
    last: NameList
}
impl Command for Name {
    fn cmds(&self) -> Vec<String> {
        own(&["name", "names"])
    }
    fn usage(&self) -> String { "[-f|-m]".to_owned() }
    fn fits(&self, size: usize) -> bool { size < 2 }
    fn auth(&self) -> i32 { 0 }

    fn run(&mut self, args: &[&str], _: &Context, _: &mut Db) -> Outcome {
        let gender = match args {
            []     => Ok(Gender::Any),
            ["-f"] => Ok(Gender::Female),
            ["-m"] => Ok(Gender::Male),
            _      => Err(InvalidArgs)
        }?;
        Ok(vec![Reply(self.gen(gender))])
    }
}

impl Name {
    pub fn build() -> IO<Self> {
        env::load();
        let conn = establish_connection();
        Ok(Self {
            female: NameList::build(&db::name_female::table.load(&conn)?)?,
            male:   NameList::build(&db::name_male::table.load(&conn)?)?,
            last:   NameList::build(&db::name_last::table.load(&conn)?)?
        })
    }

    pub fn gen(&self, gender: Gender) -> String {
        let mut rng = rand::thread_rng();
        let names   = match gender {
            Gender::Female => &self.female,
            Gender::Male   => &self.male,
            Gender::Any    => if rng.gen() { &self.female } else { &self.male }
        };
        format!("{} {}", names.choose(&mut rng), self.last.choose(&mut rng))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] #[ignore]
    fn generates_names() {
        let mut names = Name::build().unwrap();
        println!("Female: {}", names.test_def("-f").unwrap());
        println!("Male:   {}", names.test_def("-m").unwrap());
        println!("Any:    {}", names.test_def("").unwrap());
    }
}
