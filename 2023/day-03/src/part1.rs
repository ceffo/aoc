
use crate::custom_error::AocError;
use nom::{self, InputIter, InputLength, branch::alt, combinator::{value, map}, multi::many1};
use nom_locate::{position, LocatedSpan};
use quadtree_rs::{
    area::{Area, AreaBuilder},
    iter::Regions,
    point::Point,
    Quadtree,
};

// input type
type Span<'a> = LocatedSpan<&'a str>;

trait Spatial {
    fn dimensions(&self) -> (u32, u32);
}

// located type
#[derive(Debug, PartialEq, Eq)]
struct Located<A: Spatial> {
    value: A,
    position: Point<u32>,
}

impl<A> Spatial for Located<A> where A: Spatial {
    fn dimensions(&self) -> (u32, u32) {
        self.value.dimensions()
    }
}

impl<A> Located<A> where A: Spatial {
    fn area(&self) -> Option<Area<u32>> {
        AreaBuilder::default()
            .anchor(self.position)
            .dimensions(self.dimensions())
            .build()
            .ok()
    }
}

type Symbol = char;

#[derive(Debug, PartialEq, Eq)]
enum Element {
    Piece(u32),
    Symbol(Symbol),
}

type Part = Located<Element>;

impl Spatial for Element {
    fn dimensions(&self) -> (u32, u32) {
        match self {
            Element::Piece(value) => (value.to_string().len() as u32, 1),
            Element::Symbol(symbol) => (1, 1)
        }
    }
}

type SymbolQuerier = Quadtree<u32, Symbol>;

fn area_around(area: Area<u32>) -> Option<Area<u32>> {
    fn clamp_minus_one(x: u32) -> u32 { if x == 0 { 0 } else {x-1} }
    let anchor = Point {
        x: clamp_minus_one(area.anchor().x()),
        y: clamp_minus_one(area.anchor().y()),
    };
    let diff_anchor = area.anchor() - anchor;
    let dimensions = (
        area.width() + 1 + diff_anchor.x(),
        area.height() + 1 + diff_anchor.y(),
    );
    AreaBuilder::default()
        .anchor(anchor)
        .dimensions(dimensions)
        .build()
        .ok()
}

fn get_adjacent_symbols<'a>(element: &Located<Element>, symbols: &'a SymbolQuerier) -> Vec<&'a Symbol> {
    let area = element.area().unwrap();
    let area_around = area_around(area).unwrap();
    let adjacent_symbols = symbols.query(area_around);
    adjacent_symbols
        .into_iter()
        .map(|entry| entry.value_ref())
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
struct Schematics {
    pieces: Vec<Located<Element>>,
    symbols: SymbolQuerier,
}

impl Schematics {
    fn new(len: usize) -> Self {
        // assuming square schematics, we want to have a quadtree with a depth of n, allowing coordinates in the range [0, 2^n[
        let depth = (len as f64).sqrt().log2().ceil() as usize;
        Self {
            pieces: Vec::new(),
            symbols: SymbolQuerier::new(depth),
        }
    }

    fn with_piece(mut self, piece: Located<Element>) -> Self {
        self.pieces.push(piece);
        self
    }

    fn with_symbol(mut self, coordinates: Point<u32>, symbol: Symbol) -> Self {
        self.symbols.insert_pt(coordinates, symbol);
        self
    }

    fn get_engine_parts(&self) -> Vec<u32> {
        self.pieces
            .iter()
            .filter(|piece| {
                let adjacent_symbols = get_adjacent_symbols(piece, &self.symbols);
                !adjacent_symbols.is_empty()
            })
            .filter_map(|piece| {
                match piece.value {
                    Element::Piece(value) => Some(value),
                    _ => None,
                }
            })
            .collect()
    }
}

fn parse_schematics(s: Span) -> nom::IResult<Span, Schematics> {
    let size = s.input_len();
    let dot = nom::character::complete::char('.');
    let endl = nom::character::complete::line_ending;

    let parser = alt((
        map(parse_piece, Some),
        map(dot, |_| None),        
        map(endl, |_| None),
        map(parse_symbol, Some),        
    ));

    let (s, elements) = many1(parser)(s)?;
    let elements = elements.into_iter().flatten();
    let schematics = elements.fold(Schematics::new(size), |schematics, element| {
        match element.value {
            Element::Piece(piece) => schematics.with_piece(element),
            Element::Symbol(symbol) => schematics.with_symbol(element.position, symbol),
        }
    });

    Ok((s, schematics))
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String, AocError> {
    let (_, schematics) = parse_schematics(Span::new(input)).map_err(|e| AocError::ParseError(e.to_string()))?;
    let engine_parts = schematics.get_engine_parts();
    let result = engine_parts.iter().sum::<u32>().to_string();
    Ok(result)
}

fn parse_element<F>(s: Span, mut parser: F) -> nom::IResult<Span, Located<Element>>
where
    F: FnMut(Span) -> nom::IResult<Span, Element>,
{
    let (_, position) = position(s)?;
    let (s, element) = parser(s)?;
    let position: Point<_> = (position.get_column() as u32, position.location_line()).into();
    Ok((s, Located { value: element, position }))
}

fn piece_parser(s: Span) -> nom::IResult<Span, Element> {
    map(nom::character::complete::u32, Element::Piece)(s)
}

fn symbol_parser(s: Span) -> nom::IResult<Span, Element> {
    map(nom::character::complete::anychar, Element::Symbol)(s)
}

fn parse_piece(s: Span) -> nom::IResult<Span, Located<Element>> {
    parse_element(s, piece_parser)
}

fn parse_symbol(s: Span) -> nom::IResult<Span, Located<Element>> {
    parse_element(s, symbol_parser)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Part { position: (1, 1).into(), value: Element::Piece(1) }, AreaBuilder::default().anchor(Point { x: 1, y: 1 }).dimensions((1, 1)).build().unwrap())]
    #[case(Part{ position: (5, 7).into(), value: Element::Piece(1234) }, AreaBuilder::default().anchor(Point { x: 5, y: 7 }).dimensions((4, 1)).build().unwrap())]
    fn test_part_area(#[case] part: Part, #[case] expected: Area<u32>) {
        assert_eq!(part.area(), Some(expected));
    }

    #[rstest]
    #[case(
        AreaBuilder::default().anchor(Point { x: 1, y: 1 }).dimensions((1, 1)).build().unwrap(), 
        AreaBuilder::default().anchor(Point { x: 0, y: 0 }).dimensions((3, 3)).build().unwrap())]
    #[case(
        AreaBuilder::default().anchor(Point { x: 0, y: 0 }).dimensions((4, 4)).build().unwrap(), 
        AreaBuilder::default().anchor(Point { x: 0, y: 0 }).dimensions((5, 5)).build().unwrap())]
    #[case(
        AreaBuilder::default().anchor(Point { x: 0, y: 1 }).dimensions((4, 4)).build().unwrap(), 
        AreaBuilder::default().anchor(Point { x: 0, y: 0 }).dimensions((5, 6)).build().unwrap())]
    #[case(
        AreaBuilder::default().anchor(Point { x: 10, y: 10 }).dimensions((5, 5)).build().unwrap(), 
        AreaBuilder::default().anchor(Point { x: 9, y: 9 }).dimensions((7, 7)).build().unwrap())]
    fn test_area_around(#[case] area: Area<u32>, #[case] expected: Area<u32>) {
        assert_eq!(area_around(area), Some(expected));
    }

    #[rstest]
    #[case("1", Part { position: (1, 1).into(), value: Element::Piece(1) })]
    #[case("1234", Part { position: (1, 1).into(), value: Element::Piece(1234) })]
    //#[case("1234", Piece { coordinates: (1, 1), value: 1234 })]
    fn test_parse_piece(#[case] input: &str, #[case] expected: Part) {
        let result = parse_piece(Span::new(input));
        assert!(result.is_ok());
        let (remaining, piece) = result.unwrap();
        assert_eq!(*remaining.fragment(), "");
        assert_eq!(piece, expected);
    }

    #[rstest]
    #[case(".", Schematics::new(1))]
    #[case("..*", Schematics::new(3).with_symbol((3,1).into(), '*'))]
    #[case("..*.233", 
        Schematics::new(7)
        .with_symbol((3,1).into(), '*')
        .with_piece(Located { value: Element::Piece(233), position: (5,1).into() }))]
    fn test_parse_schematics(#[case] input: &str, #[case] expected: Schematics) {
        let result = dbg!(parse_schematics(Span::new(input)));
        assert!(result.is_ok());
        let (remaining, schematics) = result.unwrap();
        assert_eq!(*remaining.fragment(), "");
        assert_eq!(schematics, dbg!(expected));
    }

    #[ignore]
    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..";
        assert_eq!("4361", process(input)?);
        Ok(())
    }
}
