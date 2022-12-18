use std::collections::HashMap;

pub type Ann = HashMap<String, String>;

// #TODO consider Anned?
pub struct Annotated<T>(T, Ann);
