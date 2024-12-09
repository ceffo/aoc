use crate::custom_error::AocError;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let reports = input
        .lines()
        .map(parse_line)
        .collect::<Result<Vec<_>, _>>()?;
    let safe_reports = reports.into_iter().filter(|r| report_type(r) != ReportType::UnSafe).count();
    Ok(safe_reports.to_string())
}

fn parse_line(line: &str) -> Result<Report, AocError> {
    line.split_whitespace()
        .map(|n| n.parse::<u32>().map_err(|e| AocError::ParseError(e.to_string())))
        .collect()
}

// ReportType is either UnSafe, SafeIncreasing, or SafeDecreasing
#[derive(Debug, PartialEq)]
enum ReportType {
    UnSafe,
    SafeIncreasing,
    SafeDecreasing,
}

type Report = Vec<u32>;

fn classify_diff(diff: i32) -> ReportType {
    match diff {
        1..=3 => ReportType::SafeIncreasing,
        -3..=-1 => ReportType::SafeDecreasing,
        _ => ReportType::UnSafe,
    }
}

fn report_type(report: &Report) -> ReportType {
    let differences = differences(report);
    let report_type = classify_diff(differences[0]);
    let r = differences.into_iter().skip(1).try_fold(report_type, |acc, diff| {
        if classify_diff(diff) != acc {
            Err(())
        } else {
            Ok(acc)
        }
    });
    r.unwrap_or(ReportType::UnSafe)
}

fn differences(report: &Report) -> Vec<i32> {
    // compute the differences between consecutive numbers
    report.windows(2).map(|w| w[1] as i32 - w[0] as i32).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(vec![1, 2, 3, 4], vec![1, 1, 1])]
    #[case(vec![1, 4, 5, 6], vec![3, 1, 1])]
    #[case(vec![1, 4, 7, 10], vec![3, 3, 3])]
    fn test_differences(#[case] report: Report, #[case] expected: Vec<i32>) {
        assert_eq!(differences(&report), expected);
    }

    #[rstest]
    #[case(vec![1, 2, 3, 4], ReportType::SafeIncreasing)]
    #[case(vec![1, 3, 6, 7], ReportType::SafeIncreasing)]
    #[case(vec![4, 3, 2, 1], ReportType::SafeDecreasing)]
    #[case(vec![7, 6, 3, 1], ReportType::SafeDecreasing)]
    #[case(vec![1, 2, 3, 8], ReportType::UnSafe)]
    #[case(vec![1, 2, 2, 3], ReportType::UnSafe)]
    #[case(vec![1, 4, 3, 5], ReportType::UnSafe)]
    fn test_report_type(#[case] report: Report, #[case] expected: ReportType) {
        assert_eq!(report_type(&report), expected);
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "7 6 4 2 1
1 2 7 8 9
9 7 6 2 1
1 3 2 4 5
8 6 4 4 1
1 3 6 7 9";
        assert_eq!("2", process(input)?);
        Ok(())
    }
}
