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
    let (a, b) = transpose(pairs);
    let output = similarity(a, b).into_iter().sum::<u32>();
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

fn transpose(pairs: Vec<(u32, u32)>) -> (Vec<u32>, Vec<u32>) {
    pairs.into_iter().unzip()
}

fn similarity(a: Vec<u32>, b: Vec<u32>) -> Vec<u32> {
    // count the number of times each element in a occurs in b
    // naive implementation
    a.into_iter().map(|x| b.iter().filter(|&&y| x == y).count() as u32 * x).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(vec![1, 2, 3], vec![3, 2, 1], vec![1, 2, 3])]
    #[case(vec![1, 2, 3], vec![2, 2, 3], vec![0, 4, 3])]
    #[case(vec![1, 1, 1], vec![1, 1, 1], vec![3, 3, 3])]
    fn test_similarity(#[case] a: Vec<u32>, #[case] b: Vec<u32>, #[case] expected: Vec<u32>) {
        let actual = similarity(a, b);
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
        assert_eq!("31", process(input)?);
        Ok(())
    }
}
