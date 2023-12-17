#[allow(unused_imports)]
#[allow(unused_variables)]
#[allow(dead_code)]
#[allow(unused_assignments)]
use std::collections::BTreeMap;

use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, line_ending, space0, space1},
    combinator::{map, map_res},
    multi::{many1, many_m_n, separated_list0, separated_list1},
    sequence::{preceded, terminated},
    IResult,
};
use strum::EnumString;

use crate::custom_error::AocError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Range {
    from: u32,
    length: u32,
}

impl Range {
    fn new(from: u32, length: u32) -> Self {
        Self { from, length }
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.from <= other.from + other.length && other.from <= self.from + self.length
    }

    fn intersection(&self, other: &Self) -> Option<Self> {
        if self.overlaps(other) {
            let from = self.from.max(other.from);
            let to = (self.from + self.length).min(other.from + other.length);
            Some(Range::new(from, to - from))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RangeMapping {
    range: Range,
    to: u32,
}

impl RangeMapping {
    fn new(range: Range, to: u32) -> Self {
        Self { range, to }
    }
}

type RangeMappings = Vec<RangeMapping>;

#[tracing::instrument]
// map_range maps a range onto possibly multiple ranges
fn map_range(mappings: &RangeMappings, range: &Range) -> Vec<Range> {
    let mut result = Vec::new();
    let mut range = *range; // copy because we will consume it as we go
    for mapping in mappings {
        if let Some(intersection) = mapping.range.intersection(&range) {
            // push the unmapped part of the range
            if intersection.from > range.from {
                result.push(Range::new(range.from, intersection.from - range.from));
            }
            // push the mapped part of the intersecting range
            result.push(Range::new(
                mapping.to + intersection.from - mapping.range.from,
                intersection.length,
            ));
            // consume the mapped part of the range
            let consumed_length = intersection.from + intersection.length - range.from;
            range.from = intersection.from + intersection.length;
            range.length -= consumed_length;
        }
    }
    // push the unmapped leftover part of the range
    if range.length > 0 {
        result.push(range);
    }
    result.sort();
    result
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

type EntityMap = BTreeMap<Entity, (Entity, RangeMappings)>;

fn new_entity_map(maps: Vec<(Entity, Entity, RangeMappings)>) -> EntityMap {
    let mut entity_map = EntityMap::new();
    for (entity1, entity2, ranges) in maps {
        entity_map.insert(entity1, (entity2, ranges));
    }
    entity_map
}
#[derive(Debug)]
struct Game {
    seed_ranges: Vec<Range>,
    entity_map: EntityMap,
}

impl Game {
    fn new(seed_ranges: Vec<Range>, maps: Vec<(Entity, Entity, RangeMappings)>) -> Self {
        Self {
            seed_ranges,
            entity_map: new_entity_map(maps),
        }
    }
}

#[tracing::instrument]
fn parse_number(input: &str) -> IResult<&str, u32> {
    preceded(space0, nom::character::complete::u32)(input)
}

#[tracing::instrument]
fn parse_range(input: &str) -> IResult<&str, Range> {
    map(many_m_n(2, 2, parse_number), |numbers| {
        Range::new(numbers[0], numbers[1])
    })(input)
}

#[tracing::instrument]
fn parse_range_mapping(input: &str) -> IResult<&str, RangeMapping> {
    let (input, dest) = parse_number(input)?;
    let (input, range) = parse_range(input)?;
    Ok((input, RangeMapping::new(range, dest)))
}

#[tracing::instrument]
fn parse_entity(input: &str) -> IResult<&str, Entity> {
    map_res(alpha1, |s: &str| s.parse())(input)
}

#[tracing::instrument]
fn parse_map(input: &str) -> IResult<&str, (Entity, Entity, RangeMappings)> {
    let (input, entity1) = parse_entity(input)?;
    let (input, _) = tag("-to-")(input)?;
    let (input, entity2) = parse_entity(input)?;
    let (input, _) = terminated(tag(" map:"), line_ending)(input)?;
    let (input, mut mappings) = separated_list0(line_ending, parse_range_mapping)(input)?;
    mappings.sort_unstable_by_key(|range| range.range.from);
    Ok((input, (entity1, entity2, mappings)))
}

#[tracing::instrument]
fn parse_ranges(input: &str) -> IResult<&str, Vec<Range>> {
    separated_list1(space1, parse_range)(input)
}

#[tracing::instrument]
fn parse_game(input: &str) -> IResult<&str, Game> {
    let (input, seed_ranges) = preceded(tag("seeds: "), parse_ranges)(input)?;
    let (input, _) = line_ending(input)?;
    let (input, _) = line_ending(input)?;
    let (input, maps) = separated_list0(many1(line_ending), parse_map)(input)?;

    Ok((input, Game::new(seed_ranges, maps)))
}

#[tracing::instrument]
fn follow_map(game: &Game, entity: Entity, ranges: Vec<Range>) -> Vec<Range> {
    let (next_entity, mappings) = game.entity_map.get(&entity).unwrap();
    let ranges = ranges
        .iter()
        .flat_map(|range| map_range(mappings, range))
        .collect::<Vec<_>>();
    if *next_entity == Entity::Location {
        ranges
    } else {
        follow_map(game, *next_entity, ranges)
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String, AocError> {
    let (_, game) = parse_game(input).map_err(|e| AocError::ParseError(e.to_string()))?;
    let min_location = follow_map(&game, Entity::Seed, game.seed_ranges.clone())
        .iter()
        .min()
        .ok_or(AocError::ParseError("No range on seeds found".to_string()))
        .map(|range| range.from)?;
    Ok(min_location.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(
        // disjoint
        vec![
            RangeMapping::new(Range::new(0, 10), 100),
            RangeMapping::new(Range::new(20, 10), 200),
        ],
        Range::new(0, 30),
        vec![Range::new(10, 10), Range::new(100, 10), Range::new(200, 10),]
    )]
    #[case(
        // no overlap
        vec![
            RangeMapping::new(Range::new(0, 10), 100),
            RangeMapping::new(Range::new(20, 10), 200),
        ],
        Range::new(100, 30),
        vec![Range::new(100, 30),]
    )]
    #[case(
        // everything!
        vec![
            RangeMapping::new(Range::new(10, 10), 100),
            RangeMapping::new(Range::new(30, 10), 200),
        ],
        Range::new(0, 50),

        vec![Range::new(0,10), Range::new(20, 10), Range::new(40, 10), Range::new(100, 10), Range::new(200, 10),]
    )]
    fn test_map_range(
        #[case] mappings: RangeMappings,
        #[case] range: Range,
        #[case] expected: Vec<Range>,
    ) {
        let result = map_range(&mappings, &range);
        assert_eq!(result, expected);
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
        assert_eq!("46", process(input)?);
        Ok(())
    }
}
