use itertools::Itertools;
use nom::{
    character::complete::{line_ending, space1},
    multi::separated_list1,
    IResult, Parser,
};
use nom_supreme::{parser_ext::ParserExt, tag::complete::tag};

use crate::custom_error::AocError;

fn travel_distance(hold: u32, duration: u32) -> u32 {
    // each hold time unit gives one unit of speed for the remaining duration
    if hold >= duration {
        0 // no time left to run
    } else {
        hold * (duration - hold)
    }
}

#[derive(Debug)]
struct Run {
    time: u32,
    distance: u32,
}

impl Run {
    fn new(time: u32, distance: u32) -> Self {
        Self { time, distance }
    }

    fn ways_to_beat(&self) -> u32 {
        // count the number of ways to beat this run
        (0..self.time)
            .map(|hold| travel_distance(hold, self.time))
            .filter(|&distance| distance > self.distance)
            .count() as u32
    }
}

#[derive(Debug)]
struct Game {
    runs: Vec<Run>,
}

#[tracing::instrument]
fn parse_game(input: &str) -> IResult<&str, Game> {
    let (input, times): (&str, Vec<u32>) = separated_list1(space1, nom::character::complete::u32)
        .preceded_by(tag("Time:").precedes(space1))
        .parse(input)?;
    let (input, _) = line_ending(input)?;
    let (input, distances): (&str, Vec<u32>) =
        separated_list1(space1, nom::character::complete::u32)
            .preceded_by(tag("Distance:").precedes(space1))
            .parse(input)?;
    let runs = times
        .iter()
        .zip_eq(distances.iter())
        .map(|(time, distance)| Run::new(*time, *distance))
        .collect();
    Ok((input, Game { runs }))
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String, AocError> {
    let (_, game) = parse_game(input).map_err(|e| AocError::ParseError(e.to_string()))?;
    let result: u32 = game.runs.iter().map(|run| run.ways_to_beat()).product();
    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "Time:      7  15   30
Distance:  9  40  200";
        assert_eq!("288", process(input)?);
        Ok(())
    }
}
