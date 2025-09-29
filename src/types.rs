use crate::tui::TableData;
use ratatui::layout::Constraint;
use ratatui::widgets::Cell;

pub struct PriceIndex {
    pub asset: String,
    pub price: f64,
}

impl TableData for PriceIndex {
    fn header() -> Vec<impl Into<String>> {
        vec![
            String::from("Asset"),
            String::from("Price"),
            String::from("Asset"),
            String::from("Price"),
            String::from("Asset"),
            String::from("Price"),
            String::from("Asset"),
            String::from("Price"),
        ]
    }

    fn column_constraints() -> Vec<Constraint> {
        vec![
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
        ]
    }

    fn to_rows<'a>(data: &'a Vec<&Self>) -> Vec<ratatui::widgets::Row<'a>> {
        let mut rows = vec![];
        let mut i = 0;
        for _ in 0..data.len() / 4 {
            let cells = vec![
                Cell::from(data[i].asset.clone()),
                Cell::from(format!("{:.5}", data[i].price)),
                Cell::from(data[i + 1].asset.clone()),
                Cell::from(format!("{:.5}", data[i + 1].price)),
                Cell::from(data[i + 2].asset.clone()),
                Cell::from(format!("{:.5}", data[i + 2].price)),
                Cell::from(data[i + 3].asset.clone()),
                Cell::from(format!("{:.5}", data[i + 3].price)),
            ];
            rows.push(ratatui::widgets::Row::new(cells));
            i += 4;
        }
        if i < data.len() {
            let mut cells = vec![];
            for j in i..data.len() {
                cells.push(Cell::from(data[j].asset.clone()));
                cells.push(Cell::from(format!("{:.5}", data[j].price)));
            }
            rows.push(ratatui::widgets::Row::new(cells));
        }
        rows
    }

    fn comparator(&self, other: &Self) -> std::cmp::Ordering {
        self.asset
            .cmp(&other.asset)
            .then(self.price.partial_cmp(&other.price).unwrap())
    }
}
