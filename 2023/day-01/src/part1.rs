use nom::{character::complete::anychar, combinator::iterator, IResult};

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
fn parse_digit(input: &str) -> IResult<&str, Option<u32>> {
    let (input, digit) = anychar(input)?;
    Ok((input, digit.to_digit(10)))
}

#[tracing::instrument]
fn parser(input: &str) -> IResult<&str, Vec<u32>> {
    let mut it = iterator(input, parse_digit);
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
    #[case("1abc2", 12)]
    #[case("pqr3stu8vwx", 38)]
    #[case("a1b2c3d4e5f", 15)]
    #[case("treb7uchet", 77)]
    fn test_process_line(#[case] input: &str, #[case] expected: u32) {
        assert_eq!(expected, process_line(input).unwrap());
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "\
1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet";
        assert_eq!("142", process(input)?);
        Ok(())
    }
}
