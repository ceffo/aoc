use miette::miette;
use nom::{
    character::complete::{newline, space1},
    multi::separated_list1,
    IResult,
};

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, reports) =
        parse(input).map_err(|e| miette!("error parsing input: {}", e.to_string()))?;
    let safe_count = reports.iter().filter(|r| report_type(r).is_safe()).count();
    Ok(safe_count.to_string())
}

fn parse(input: &str) -> IResult<&str, Vec<Report>> {
    separated_list1(
        newline,
        separated_list1(space1, nom::character::complete::u32),
    )(input)
}

#[derive(Debug, PartialEq)]
enum ReportType {
    UnSafe,
    SafeIncreasing,
    SafeDecreasing,
}

impl ReportType {
    fn is_safe(&self) -> bool {
        matches!(
            self,
            ReportType::SafeIncreasing | ReportType::SafeDecreasing
        )
    }
}

type Report = Vec<u32>;

fn report_type(report: &Report) -> ReportType {
    one_removed(report)
        .map(|r| classify_differences(&differences(&r)))
        .find(ReportType::is_safe)
        .unwrap_or(ReportType::UnSafe)
}

fn differences(report: &Report) -> Vec<i32> {
    report
        .windows(2)
        .map(|w| w[1] as i32 - w[0] as i32)
        .collect()
}

fn classify_differences(differences: &[i32]) -> ReportType {
    let report_type = classify_difference(differences[0]);
    differences
        .iter()
        .skip(1)
        .try_fold(report_type, |acc, &diff| match classify_difference(diff) {
            t if t == acc => Ok(acc),
            _ => Err(()),
        })
        .unwrap_or(ReportType::UnSafe)
}

fn classify_difference(diff: i32) -> ReportType {
    match diff {
        1..=3 => ReportType::SafeIncreasing,
        -3..=-1 => ReportType::SafeDecreasing,
        _ => ReportType::UnSafe,
    }
}

/// Generate all possible reports by removing one element from the input
fn one_removed<T: Copy>(d: &[T]) -> impl Iterator<Item = Vec<T>> + '_ {
    (0..d.len()).map(move |i| {
        let mut r = d.to_owned();
        r.remove(i);
        r
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(vec![1], vec![vec![]])]
    #[case(vec![1, 2, 3, 4], vec![vec![2, 3, 4], vec![1, 3, 4], vec![1, 2, 4], vec![1, 2, 3]])]
    fn test_one_removed(#[case] input: Vec<u32>, #[case] expected: Vec<Vec<u32>>) {
        let res: Vec<Vec<u32>> = one_removed(&input).collect();
        assert_eq!(res, expected);
    }

    #[rstest]
    #[case(vec![1, 2, 3, 4], vec![1, 1, 1])]
    #[case(vec![1, 4, 5, 6], vec![3, 1, 1])]
    #[case(vec![1, 4, 7, 10], vec![3, 3, 3])]
    fn test_differences(#[case] report: Report, #[case] expected: Vec<i32>) {
        assert_eq!(differences(&report), expected);
    }

    #[rstest]
    #[case("7 6 4 2 1", ReportType::SafeDecreasing)]
    #[case("1 2 7 8 9", ReportType::UnSafe)]
    #[case("9 7 6 2 1", ReportType::UnSafe)]
    #[case("1 3 2 4 5", ReportType::SafeIncreasing)]
    #[case("8 6 4 4 1", ReportType::SafeDecreasing)]
    #[case("1 3 6 7 9", ReportType::SafeIncreasing)]
    fn test_report_type(#[case] input: &str, #[case] expected: ReportType) {
        let report = &parse(input).unwrap().1[0];
        assert_eq!(report_type(report), expected);
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9";
        assert_eq!("4", process(input)?);
        Ok(())
    }
}
