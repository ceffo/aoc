use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map_res, value},
    multi::separated_list1,
    IResult,
};

use crate::custom_error::AocError;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
enum Color {
    Red,
    Blue,
    Green,
}

type Draw = HashMap<Color, u32>;

struct Bag {
    cubes: HashMap<Color, u32>,
}

impl Bag {
    fn new() -> Self {
        Self {
            cubes: HashMap::new(),
        }
    }
    fn add(&mut self, color: Color, count: u32) -> &mut Self {
        let entry = self.cubes.entry(color).or_insert(0);
        *entry += count;
        self
    }

    // can_draw returns true if the bag has enough cubes to draw the given draw
    fn can_draw(&self, draw: &Draw) -> bool {
        for (color, count) in draw {
            if let Some(bag_count) = self.cubes.get(color) {
                if bag_count < count {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Debug)]
struct Game {
    id: u32,
    draws: Vec<Draw>,
}

impl Game {
    fn impossible_draw(&self, bag: &Bag) -> Option<usize> {
        for (i, draw) in self.draws.iter().enumerate() {
            if !bag.can_draw(draw) {
                return Some(i);
            }
        }
        None
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String, AocError> {
    let mut bag = Bag::new();
    let bag = bag
        .add(Color::Red, 12)
        .add(Color::Blue, 14)
        .add(Color::Green, 13);
    let games = parse(input)?;
    let impossible_games = games
        .iter()
        .filter(|game| game.impossible_draw(bag).is_none())
        .map(|game| game.id)
        .collect::<Vec<_>>();
    Ok(impossible_games.iter().sum::<u32>().to_string())
}

fn parse(input: &str) -> miette::Result<Vec<Game>, AocError> {
    let games = input
        .lines()
        .map(parse_line)
        .collect::<miette::Result<Vec<_>, _>>()?;
    Ok(games)
}

fn parse_game(input: &str) -> IResult<&str, Game> {
    let (input, _) = tag("Game ")(input)?;
    let (input, id) = map_res(digit1, str::parse)(input)?;
    let (input, _) = tag(": ")(input)?;
    let (input, draws) = parse_draws(input)?;
    Ok((input, Game { id, draws }))
}

fn parse_count_color(input: &str) -> IResult<&str, (Color, u32)> {
    let mut parse_color = alt((
        value(Color::Red, tag(" red")),
        value(Color::Blue, tag(" blue")),
        value(Color::Green, tag(" green")),
    ));
    let (input, count) = map_res(digit1, str::parse)(input)?;
    let (input, color) = parse_color(input)?;
    Ok((input, (color, count)))
}

fn parse_draw(input: &str) -> IResult<&str, Draw> {
    let (input, draws) = separated_list1(tag(", "), parse_count_color)(input)?;
    let mut draw = Draw::new();
    for (color, count) in draws {
        draw.insert(color, count);
    }
    Ok((input, draw))
}

fn parse_draws(input: &str) -> IResult<&str, Vec<Draw>> {
    let (input, draws) = separated_list1(tag("; "), parse_draw)(input)?;
    Ok((input, draws))
}

fn parse_line(line: &str) -> miette::Result<Game, AocError> {
    let (_, game) = parse_game(line).expect("cannot parse game");
    Ok(game)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("3 red", (Color::Red, 3))]
    #[case("4 blue", (Color::Blue, 4))]
    #[case("11 green", (Color::Green, 11))]
    fn test_parse_count_color(#[case] input: &str, #[case] expected: (Color, u32)) {
        let (_, result) = parse_count_color(input).expect("failed to parse count color");
        assert_eq!(expected, result);
    }

    #[rstest]
    #[case("3 red", Draw::from([(Color::Red, 3)]))]
    #[case("3 red, 4 green", Draw::from([(Color::Red, 3), (Color::Green, 4)]))]
    #[case("3 red, 4 green, 17 blue", Draw::from([(Color::Red, 3), (Color::Green, 4), (Color::Blue, 17)]))]
    fn test_parse_draw(#[case] input: &str, #[case] expected: Draw) {
        let (_, result) = parse_draw(input).expect("failed to parse draw");
        assert_eq!(expected, result);
    }

    #[rstest]
    #[case("3 red; 4 green", vec![Draw::from([(Color::Red, 3)]), Draw::from([(Color::Green, 4)])])]
    #[case("3 red; 4 green; 17 blue", vec![Draw::from([(Color::Red, 3)]), Draw::from([(Color::Green, 4)]), Draw::from([(Color::Blue, 17)])])]
    #[case("3 red, 4 green; 17 blue", vec![Draw::from([(Color::Red, 3), (Color::Green, 4)]), Draw::from([(Color::Blue, 17)])])]
    fn test_parse_draws(#[case] input: &str, #[case] expected: Vec<Draw>) {
        let (_, result) = parse_draws(input).expect("failed to parse draws");
        assert_eq!(expected, result);
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";
        assert_eq!("8", process(input)?);
        Ok(())
    }
}
