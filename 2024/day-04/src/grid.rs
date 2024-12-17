use miette::miette;

pub struct Grid<T> {
    pub cells: Vec<T>,
    pub width: usize,
    pub height: usize,
    columns: Vec<Vec<T>>,
    diagonals: Vec<Vec<T>>,
}

impl<T: Clone> Grid<T> {
    pub fn new(cells: Vec<T>, width: usize) -> miette::Result<Self> {
        if cells.len() % width != 0 {
            return Err(miette!("data length is not a multiple of width"));
        }
        let height = cells.len() / width;
        let columns = Self::get_columns(&cells, width);
        let diagonals = Self::get_diagonals(&cells, width);
        Ok(Self {
            cells,
            width,
            height,
            columns,
            diagonals,
        })
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        Self::at(&self.cells, x, y, self.width)
    }

    pub fn rows(&self) -> impl Iterator<Item = &[T]> {
        self.cells.chunks(self.width)
    }

    pub fn columns(&self) -> impl Iterator<Item = &[T]> {
        self.columns.iter().map(|c| c.as_slice())
    }

    pub fn diagonals(&self) -> impl Iterator<Item = &[T]> {
        self.diagonals.iter().map(|d| d.as_slice())
    }

    fn at(data: &[T], x: usize, y: usize, width: usize) -> Option<&T> {
        if x < width && y < data.len() / width {
            Some(&data[y * width + x])
        } else {
            None
        }
    }

    fn get_columns(data: &[T], width: usize) -> Vec<Vec<T>> {
        let height = data.len() / width;
        (0..width)
            .map(|x| {
                (0..height)
                    .map(|y| Self::at(data, x, y, width).unwrap().clone())
                    .collect()
            })
            .collect()
    }

    pub fn get_diagonals(data: &[T], width: usize) -> Vec<Vec<T>> {
        let height = data.len() / width;
        let num_diagonals = width + height - 1;
        let mut diagonals = Vec::with_capacity(num_diagonals * 2);

        // Top-left to bottom-right diagonals
        for k in 0..num_diagonals {
            let mut diagonal = Vec::new();
            for i in 0..=k {
                if i < height && k - i < width {
                    if let Some(value) = Self::at(data, k - i, i, width) {
                        diagonal.push(value.clone());
                    }
                }
            }
            if !diagonal.is_empty() {
                diagonals.push(diagonal);
            }
        }

        // Top-right to bottom-left diagonals
        for k in 0..num_diagonals {
            let mut diagonal = Vec::new();
            for i in 0..=k {
                if i < height && k - i < width {
                    if let Some(value) = Self::at(data, width - 1 - (k - i), i, width) {
                        diagonal.push(value.clone());
                    }
                }
            }
            if !diagonal.is_empty() {
                diagonals.push(diagonal);
            }
        }
        diagonals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() -> miette::Result<()> {
        let grid = Grid::new(vec![1, 2, 3, 4], 2)?;
        assert_eq!(grid.cells, vec![1, 2, 3, 4]);
        assert_eq!(grid.width, 2);
        assert_eq!(grid.height, 2);
        Ok(())
    }

    #[test]
    fn test_get() -> miette::Result<()> {
        let grid = Grid::new(vec![1, 2, 3, 4], 2)?;
        assert_eq!(grid.get(0, 0), Some(&1));
        assert_eq!(grid.get(1, 0), Some(&2));
        assert_eq!(grid.get(0, 1), Some(&3));
        assert_eq!(grid.get(1, 1), Some(&4));
        assert_eq!(grid.get(2, 0), None);
        assert_eq!(grid.get(0, 2), None);
        Ok(())
    }

    #[test]
    fn test_rows() -> miette::Result<()> {
        let grid = Grid::new(vec![1, 2, 3, 4], 2)?;
        let rows: Vec<_> = grid.rows().map(|row| row.to_vec()).collect();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows, vec![vec![1, 2], vec![3, 4]]);
        Ok(())
    }

    #[test]
    fn test_columns() -> miette::Result<()> {
        let grid = Grid::new((1..=6).collect(), 3)?;
        let columns = grid.columns().collect::<Vec<_>>();
        assert_eq!(columns.len(), 3);
        assert_eq!(columns, vec![vec![1, 4], vec![2, 5], vec![3, 6]]);
        Ok(())
    }

    #[test]
    fn test_all_diagonals() -> miette::Result<()> {
        let grid = Grid::new((1..=9).collect(), 3)?;
        let diagonals = grid.diagonals().collect::<Vec<_>>();
        assert_eq!(diagonals.len(), 10);
        assert_eq!(
            diagonals,
            vec![
                vec![1],
                vec![2, 4],
                vec![3, 5, 7],
                vec![6, 8],
                vec![9],
                vec![3],
                vec![2, 6],
                vec![1, 5, 9],
                vec![4, 8],
                vec![7]
            ]
        );
        Ok(())
    }
}
