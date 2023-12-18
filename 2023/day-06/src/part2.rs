use nom::{
    character::complete::{line_ending, space1},
    multi::separated_list1,
    IResult, Parser,
};
use nom_supreme::{parser_ext::ParserExt, tag::complete::tag};

use crate::custom_error::AocError;

#[derive(Debug)]
struct Run {
    time: u64,
    distance: u64,
}

impl Run {
    fn new(time: u64, distance: u64) -> Self {
        Self { time, distance }
    }

    #[tracing::instrument]
    fn ways_to_beat(&self) -> Option<u64> {
        // the distance follows a quadratic function of the hold time
        // d(h) = h * (t - h)
        // so we just need to find the roots of the quadratic equation
        // -h^2 + t * h - d = 0
        // h = (t +/- sqrt(t^2 - 4 * d)) / 2
        // this gives us the two hold times that give the same distance
        // the number of ways to beat the run is the number of hold times between these two values
        let t = self.time as f64;
        let d = self.distance as f64;
        let t2 = t * t;
        if t2 < 4.0 * d {
            None
        } else {
            let sqrt_discriminant = (t2 - 4.0 * d).sqrt();
            let h1 = (t - sqrt_discriminant) / 2.0;
            let h2 = (t + sqrt_discriminant) / 2.0;
            let h1 = (h1 + 1.0).floor() as u64; // we need to round up to the next integer to get the first hold time that gives a greater distance
            let h2 = (h2 - 1.0).ceil() as u64; // we need to round down to the previous integer to get the last hold time that gives a greater distance
            Some(h2 - h1 + 1)
        }
    }
}

#[derive(Debug)]
struct Game {
    run: Run,
}

fn next_multipler(n: u64) -> u64 {
    // return the power of 10 that is next
    // e.g. 1 -> 10, 10 -> 100, 100 -> 1000
    let mut m = 1;
    while m <= n {
        m *= 10;
    }
    m
}

fn concatenate_int(ts: Vec<u64>) -> u64 {
    ts.into_iter()
        .rev()
        .fold((0, 1), |(acc, mul), t| {
            (acc + t * mul, mul * next_multipler(t))
        })
        .0
}

#[tracing::instrument]
fn parse_game(input: &str) -> IResult<&str, Game> {
    let (input, time) = separated_list1(space1, nom::character::complete::u64)
        .preceded_by(tag("Time:").precedes(space1))
        .map(concatenate_int)
        .parse(input)?;
    let (input, _) = line_ending(input)?;
    let (input, distance) = separated_list1(space1, nom::character::complete::u64)
        .preceded_by(tag("Distance:").precedes(space1))
        .map(concatenate_int)
        .parse(input)?;

    let run = Run::new(time, distance);
    Ok((input, Game { run }))
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String, AocError> {
    let (_, game) = parse_game(input).map_err(|e| AocError::ParseError(e.to_string()))?;
    let result: u64 = game
        .run
        .ways_to_beat()
        .expect("there should always be a way to beat the run");
    Ok(result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test]
    fn test_process() -> miette::Result<()> {
        let input = "Time:      7  15   30
Distance:  9  40  200";
        assert_eq!("71503", process(input)?);
        Ok(())
    }

    #[rstest::rstest]
    #[case(vec![1], 1)]
    #[case(vec![1, 2], 12)]
    #[case(vec![246, 1441, 1012, 1111], 246144110121111)]
    fn test_concatenate_int(#[case] ts: Vec<u64>, #[case] expected: u64) {
        assert_eq!(expected, concatenate_int(ts));
    }
}
