use super::super::domain::entities::{Pokemon, PokemonName, PokemonNumber, PokemonTypes};
enum Insert {
    Ok(PokemonNumber),
    Conflict,
    Error,
}

pub trait Repositories {
    fn insert(&mut self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Insert;
}

pub struct InMemoryRespository {
    error: bool,
    pokemons: Vec<Pokemon>,
}

impl InMemoryRespository {
    pub fn new() -> Self {
        let pokemons = vec![];
        Self {
            error: false,
            pokemons: pokemons,
        }
    }

    pub fn with_error(self)->Self{
        Self{
            error:true,
            ..self
        }
    }
    
}

impl Repositories for InMemoryRespository{
    fn insert(&mut self, number: PokemonNumber, name: PokemonName, types: PokemonTypes) -> Insert {
        if self.error {
            return Insert::Error;
        }

        if self.pokemons.iter().any(|pokemon| pokemon.number == number) {
            return Insert::Conflict;
        }

        let number_clone = number.clone();
        self.pokemons.push(Pokemon::new(number_clone, name, types));
        Insert::Ok(number)
    }
}
