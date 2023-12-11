#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(dead_code)]
#[allow(unused_assignments)]
use std::collections::BTreeMap;

use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, line_ending, space0},
    combinator::{map, map_res},
    multi::{many1, many_m_n, separated_list0},
    sequence::{preceded, terminated},
    IResult,
};
use strum::EnumString;

use crate::custom_error::AocError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Range {
    from: u32,
    length: u32,
    to: u32,
}

type Ranges = Vec<Range>;

fn search_range(range_map: &Ranges, value: u32) -> Option<&Range> {
    fn range_matches_value(range: &Range, value: u32) -> Option<&Range> {
        if range.from <= value && value <= range.from + range.length {
            Some(range)
        } else {
            None
        }
    }
    let lower_bound = range_map.binary_search_by(|range| range.from.cmp(&value));
    let index = match lower_bound {
        Ok(index) => Some(index),
        Err(index) => {
            if index == 0 {
                None
            } else {
                Some(index - 1)
            }
        }
    }?;
    range_matches_value(&range_map[index], value)
}

fn convert_value(range_map: &Ranges, value: u32) -> u32 {
    if let Some(range) = search_range(range_map, value) {
        range.to + (value - range.from)
    } else {
        value
    }
}

#[derive(EnumString, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[strum(serialize_all = "lowercase")]
enum Entity {
    Seed,
    Soil,
    Fertilizer,
    Water,
    Light,
    Temperature,
    Humidity,
    Location,
}

type EntityMap = BTreeMap<Entity, (Entity, Ranges)>;

fn new_entity_map(maps: Vec<(Entity, Entity, Ranges)>) -> EntityMap {
    let mut entity_map = EntityMap::new();
    for (entity1, entity2, ranges) in maps {
        entity_map.insert(entity1, (entity2, ranges));
    }
    entity_map
}
#[derive(Debug)]
struct Game {
    seeds: Vec<u32>,
    entity_map: EntityMap,
}

impl Game {
    fn new(seeds: Vec<u32>, maps: Vec<(Entity, Entity, Ranges)>) -> Self {
        Self {
            seeds,
            entity_map: new_entity_map(maps),
        }
    }
}

fn parse_number(input: &str) -> IResult<&str, u32> {
    preceded(space0, nom::character::complete::u32)(input)
}

fn parse_numbers(input: &str) -> IResult<&str, Vec<u32>> {
    many1(parse_number)(input)
}

fn parse_range(input: &str) -> IResult<&str, Range> {
    map(many_m_n(3, 3, parse_number), |numbers| Range {
        from: numbers[1],
        length: numbers[2],
        to: numbers[0],
    })(input)
}

fn parse_entity(input: &str) -> IResult<&str, Entity> {
    map_res(alpha1, |s: &str| s.parse())(input)
}

fn parse_map(input: &str) -> IResult<&str, (Entity, Entity, Ranges)> {
    let (input, entity1) = parse_entity(input)?;
    let (input, _) = tag("-to-")(input)?;
    let (input, entity2) = parse_entity(input)?;
    let (input, _) = terminated(tag(" map:"), line_ending)(input)?;
    let (input, mut ranges) = separated_list0(line_ending, parse_range)(input)?;
    ranges.sort_unstable_by_key(|range| range.from);
    Ok((input, (entity1, entity2, ranges)))
}

fn parse_game(input: &str) -> IResult<&str, Game> {
    let (input, seeds) = preceded(tag("seeds:"), parse_numbers)(input)?;
    let (input, _) = line_ending(input)?;
    let (input, _) = line_ending(input)?;
    let (input, maps) = separated_list0(many1(line_ending), parse_map)(input)?;

    Ok((input, Game::new(seeds, maps)))
}

fn follow_map(game: &Game, entity: Entity, value: u32) -> u32 {
    let (next_entity, ranges) = game.entity_map.get(&entity).unwrap();
    let value = convert_value(ranges, value);
    if *next_entity == Entity::Location {
        value
    } else {
        follow_map(game, *next_entity, value)
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String, AocError> {
    let (_, game) = parse_game(input).map_err(|e| AocError::ParseError(e.to_string()))?;
    let min_location = game
        .seeds
        .iter()
        .map(|seed| follow_map(&game, Entity::Seed, *seed))
        .min()
        .ok_or(AocError::ParseError("No seeds found".to_string()))?;
    Ok(min_location.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        vec![
            Range {
                from: 0,
                length: 10,
                to: 10
            },
        ],
        5,
        Some(&Range {
            from: 0,
            length: 10,
            to: 10
        })
    )]
    #[case(
        vec![
            Range {
                from: 1,
                length: 10,
                to: 10
            },
        ],
        0,
        None
    )]
    #[case(
        vec![
            Range {
                from: 0,
                length: 10,
                to: 10
            },
            Range {
                from: 30,
                length: 10,
                to: 10
            },
        ],
        20,
        None
    )]
    fn test_search_range(
        #[case] range_map: Ranges,
        #[case] value: u32,
        #[case] expected: Option<&Range>,
    ) {
        assert_eq!(search_range(&range_map, value), expected);
    }

    #[rstest]
    #[case(
        vec![
            Range {
                from: 0,
                length: 10,
                to: 10
            },
        ],
        5,
        15
    )]
    #[case(
        vec![
            Range {
                from: 80,
                length: 10,
                to: 10
            },
        ],
        42,
        42
    )]
    #[case(
        vec![
            Range {
                from: 0,
                length: 10,
                to: 10
            },
            Range {
                from: 30,
                length: 10,
                to: 50
            },
        ],
        20,
        20
    )]
    #[case(
        vec![
            Range {
                from: 0,
                length: 10,
                to: 10
            },
            Range {
                from: 30,
                length: 10,
                to: 50
            },
        ],
        32,
        52
    )]
    fn test_convert(#[case] range_map: Ranges, #[case] value: u32, #[case] expected: u32) {
        assert_eq!(convert_value(&range_map, value), expected);
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4";
        assert_eq!("35", process(input)?);
        Ok(())
    }
}
