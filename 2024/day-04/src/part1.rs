use miette::miette;
use nom::{
    character::complete::{alpha1, newline},
    error::{ErrorKind, FromExternalError},
    multi::separated_list1,
    IResult,
};

use crate::grid::*;

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, grid) = grid(input).map_err(|e| miette!("failed to parse grid: {}", e))?;
    let search_terms = ["XMAS", "SAMX"];
    let count = count_occurences(&grid, &search_terms);
    Ok(count.to_string())
}

type WordSearch = Grid<char>;

fn grid(input: &str) -> IResult<&str, WordSearch> {
    let (input, rows) = separated_list1(newline, alpha1)(input)?;
    let width = rows[0].len();
    let cells: Vec<char> = rows.into_iter().flat_map(|s| s.chars()).collect();
    let grid = Grid::new(cells, width).map_err(|e| {
        nom::Err::Error(nom::error::Error::from_external_error(
            input,
            ErrorKind::Fail,
            e,
        ))
    })?;
    Ok((input, grid))
}

fn count_occurences(grid: &WordSearch, search_terms: &[&str]) -> usize {
    grid.rows()
        .chain(grid.columns())
        .chain(grid.diagonals())
        .map(|part| count_windows_matches(part, search_terms))
        .sum()
}

fn count_windows_matches(input: &[char], search_terms: &[&str]) -> usize {
    let windows = input.windows(search_terms[0].len());
    let strings = windows.map(|w| w.iter().collect::<String>());
    strings
        .filter(|s| search_terms.contains(&s.as_str()))
        .count()
}

#[cfg(test)]
#[path = "grid.rs"]
mod grid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX";
        assert_eq!("18", process(input)?);
        Ok(())
    }
}

// implement Debut
impl std::fmt::Debug for WordSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}x{}", self.width, self.height)?;
        for row in self.rows() {
            for cell in row {
                write!(f, "{}", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
