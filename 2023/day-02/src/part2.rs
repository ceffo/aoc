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

#[derive(Debug)]
struct Bag {
    cubes: HashMap<Color, u32>,
}

impl Bag {
    fn new() -> Self {
        Self {
            cubes: HashMap::new(),
        }
    }

    fn cover(mut self, draw: &Draw) -> Self {
        for (color, count) in draw {
            let entry = self.cubes.entry(*color).or_insert(0);
            *entry = *count.max(entry);
        }
        self
    }

    fn power_set(&self) -> u32 {
        self.cubes.values().product()
    }
}

#[derive(Debug)]
struct Game {
    draws: Vec<Draw>,
}

impl Game {
    fn smallest_bag(&self) -> Bag {
        self.draws
            .iter()
            .fold(Bag::new(), |bag, draw| bag.cover(draw))
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String, AocError> {
    let games = parse(input)?;
    let smallest_bags = games.iter().map(|game| game.smallest_bag());
    let power_sets = smallest_bags.map(|bag| bag.power_set());
    Ok(power_sets.sum::<u32>().to_string())
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
    let (input, _) = map_res(digit1, str::parse::<u32>)(input)?;
    let (input, _) = tag(": ")(input)?;
    let (input, draws) = parse_draws(input)?;
    Ok((input, Game { draws }))
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

    #[test]
    fn test_smallest_bag() {
        let game = Game {
            draws: vec![
                Draw::from([(Color::Blue, 3), (Color::Red, 4)]),
                Draw::from([(Color::Red, 1), (Color::Green, 2), (Color::Blue, 6)]),
                Draw::from([(Color::Green, 2)]),
            ],
        };
        let bag = game.smallest_bag();
        assert_eq!(bag.cubes[&Color::Red], 4);
        assert_eq!(bag.cubes[&Color::Blue], 6);
        assert_eq!(bag.cubes[&Color::Green], 2);
    }

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
        assert_eq!("2286", process(input)?);
        Ok(())
    }
}
