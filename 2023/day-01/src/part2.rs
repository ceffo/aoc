use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::anychar,
    combinator::{iterator, value},
    IResult,
};

use crate::custom_error::AocError;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String, AocError> {
    let output = input
        .lines()
        .map(process_line)
        .sum::<miette::Result<u32, AocError>>()?;
    Ok(output.to_string())
}

#[tracing::instrument]
fn maybe_digit(input: &str) -> IResult<&str, Option<u32>> {
    let (rest, c) = anychar(input)?;
    Ok((rest, c.to_digit(10)))
}

#[tracing::instrument]
fn parse_number(input: &str) -> IResult<&str, Option<u32>> {
    let res: IResult<&str, u32> = alt((
        value(1, tag("one")),
        value(2, tag("two")),
        value(3, tag("three")),
        value(4, tag("four")),
        value(5, tag("five")),
        value(6, tag("six")),
        value(7, tag("seven")),
        value(8, tag("eight")),
        value(9, tag("nine")),
    ))(input);
    // convert the result to an Option, dropping the remaining input
    let digit_from_letters = res.ok().map(|(_, d)| d);
    // consume the next digit, if any
    let (input, digit) = maybe_digit(input)?;
    // combine the two results
    Ok((input, digit_from_letters.or(digit)))
}

#[tracing::instrument]
fn parser(input: &str) -> IResult<&str, Vec<u32>> {
    let mut it = iterator(input, parse_number);
    let output = it.flatten().collect();
    let (input, _) = it.finish()?;
    Ok((input, output))
}

#[tracing::instrument]
pub fn process_line(line: &str) -> miette::Result<u32, AocError> {
    let (_, result) = parser(line).expect("failed to parse line");
    let mut it = result.iter();
    let first = it.next().expect("must have at least one digit");
    let last = it.last().unwrap_or(first);
    let result = first * 10 + last;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case("two1nine", 29)]
    #[case("eightwothree", 83)]
    #[case("4oneight", 48)] // overlapping case
    fn test_process_line(#[case] input: &str, #[case] expected: u32) {
        assert_eq!(expected, process_line(input).unwrap());
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "\
two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen";
        assert_eq!("281", process(input)?);
        Ok(())
    }
}
