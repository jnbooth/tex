use diesel::prelude::*;
use diesel::pg::PgConnection;
use rand::distributions::{Distribution, WeightedError, WeightedIndex};
use rand::Rng;

use crate::IO;
use crate::util::Gender;

use super::model::Name;
use super::schema::name_male;
use super::schema::name_female;
use super::schema::name_last;

#[derive(Debug, Clone)]
struct NameList {
    names: Vec<String>,
    dist: WeightedIndex<i32>
}
impl NameList {
    pub fn new(names: Vec<Name>) -> Result<NameList, WeightedError> {
        Ok(NameList { 
            names:  names.clone().into_iter().map(|x| x.name).collect(),
            dist: WeightedIndex::new(names.into_iter().map(|x| x.frequency))?
        })
    }
    pub fn choose<T: Rng>(&self, rng: &mut T) -> String {
        self.names[self.dist.sample(rng)].to_owned()
    }
}

#[derive(Debug, Clone)]
pub struct Names {
    male: NameList,
    female: NameList,
    last: NameList
}
impl Names {
    pub fn new(conn: &PgConnection) -> IO<Self> {
        Ok(Names {
            female: NameList::new(name_female::table.load(conn)?)?,
            male: NameList::new(name_male::table.load(conn)?)?,
            last: NameList::new(name_last::table.load(conn)?)?
        })
    }

    pub fn gen(&self, gender: Gender) -> String {
        let mut rng = rand::thread_rng();
        let names = match gender {
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
    use super::super::establish_connection;

    #[test]
    fn test_gen() {
        dotenv::dotenv().unwrap();
        let conn = establish_connection();
        let names = Names::new(&conn).unwrap();
        println!("Female: {}", names.gen(Gender::Female));
        println!("Male:   {}", names.gen(Gender::Male));
        println!("Any:    {}", names.gen(Gender::Any));
    }
}
