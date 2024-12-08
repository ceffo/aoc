use itertools::sorted;
use nom::{
    character::complete::line_ending,
    character::complete::digit1,
    character::complete::space1,
    combinator::map_res,
    multi::separated_list1,
    IResult,
};

use crate::custom_error::AocError;

#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String, AocError> {
    let input = _input.trim();
    let (_, pairs) = parse_input(input).map_err(|e| AocError::ParseError(e.to_string()))?;
    let output = distances(pairs).iter().sum::<u32>();
    Ok(output.to_string())
}

fn parse_int_pair(input: &str) -> IResult<&str, (u32, u32)> {
    let (input, a) = map_res(digit1, str::parse)(input)?;
    let (input, _) = space1(input)?;
    let (input, b) = map_res(digit1, str::parse)(input)?;
    Ok((input, (a, b)))
}

fn parse_input(input: &str) -> IResult<&str, Vec<(u32, u32)>> {
    separated_list1(line_ending, parse_int_pair)(input)
}

fn tuple_distance((a,b): (u32, u32)) -> u32 {
    a.abs_diff(b)
}

fn distances(pairs: Vec<(u32, u32)>) -> Vec<u32> {
    let (a, b): (Vec<u32>, Vec<u32>) = pairs.into_iter().unzip();
    let (sa, sb) = (sorted(a), sorted(b));
    sa.zip(sb).map(tuple_distance).collect()        
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("3   4", Ok(("", (3, 4))))]
    #[case("23   34", Ok(("", (23, 34))))]
    fn test_parse_pair(#[case] input: &str, #[case] expected: IResult<&str, (u32, u32)>) {
        let actual = parse_int_pair(input);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case("3   4\n4   3\n2   5\n1   3\n3   9\n3   3", Ok(("", vec![(3, 4), (4, 3), (2, 5), (1, 3), (3, 9), (3, 3)])))]
    fn test_parse_input(#[case] input: &str, #[case] expected: IResult<&str, Vec<(u32, u32)>>) {
        let actual = parse_input(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "\
3   4
4   3
2   5
1   3
3   9
3   3";
        assert_eq!("11", process(input)?);
        Ok(())
    }

}

