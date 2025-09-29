use chrono::{Local, TimeZone, Utc};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use comfy_table::{Cell, Table};
use hyperliquid_rust_sdk::{
    AssetPosition, OpenOrdersResponse, UserStateResponse, UserTokenBalance,
};

use crate::types::PriceIndex;

pub fn line() {
    println!("--------------------------------------------------------------------------------");
}

pub fn draw_all_mids_table(mids: &Vec<PriceIndex>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    println!("All Tickers Mids:");
    println!("                   Asset     ");
    println!("              ---------------");
    println!("                   Price($)  ");
    line();
    let mut row1 = vec![];
    let mut row2 = vec![];
    for t in mids {
        if !t.asset.starts_with("@") {
            row1.push(
                Cell::from(t.asset.clone())
                    .set_alignment(CellAlignment::Center)
                    .add_attributes(vec![Attribute::Bold]),
            );
            row2.push(
                Cell::from(format!("{:.5}", t.price))
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::DarkRed),
            );
        }
        if row1.len() >= 8 {
            table.add_row(row1.clone());
            table.add_row(row2.clone());
            row1.clear();
            row2.clear();
        }
    }
    if row1.len() > 0 {
        table.add_row(row1);
        table.add_row(row2);
    }
    println!("{table}");
}

pub fn draw_user_positions_table(positions: Vec<AssetPosition>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        "Type",
        "Asset",
        "Entry Price",
        "Leverage",
        "Liquidation",
        "Margin Used",
        "Position Value",
        "Return on Equity",
        "Quantity",
        "Unrealized PnL",
    ]);

    positions.iter().for_each(|p| {
        table.add_row(vec![
            &p.type_string,
            &p.position.coin,
            p.position.entry_px.as_ref().unwrap_or(&"-".to_string()),
            &format!("{}", p.position.leverage.value),
            p.position
                .liquidation_px
                .as_ref()
                .unwrap_or(&"-".to_string()),
            &p.position.margin_used,
            &p.position.position_value,
            &p.position.return_on_equity,
            &p.position.szi,
            &p.position.unrealized_pnl,
        ]);
    });
    println!("{table}");
}

pub fn draw_orders_table(orders: Vec<OpenOrdersResponse>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        "Asset",
        "Limit Price",
        "OrderID",
        "Side",
        "Quantity",
        "Time",
    ]);

    orders.iter().for_each(|o| {
        let date_time = Utc.timestamp_millis_opt(o.timestamp as i64).unwrap();
        let time = date_time
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        table.add_row(vec![
            &o.coin,
            &o.limit_px,
            &format!("{}", o.oid),
            &o.side,
            &o.sz,
            &time,
        ]);
    });
    println!("{table}");
}

pub fn draw_balance_table(
    state: UserStateResponse,
    tokens: Vec<UserTokenBalance>,
    all_details: bool,
) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    // Spot Token Balance
    println!("Spot Tokens:");
    table.set_header(vec!["Asset", "Hold", "Total", "Average Cost", "PnL"]);
    // TODO: Implement average cost calculation
    tokens.iter().for_each(|token| {
        table.add_row(vec![&token.coin, &token.hold, &token.total, "0.0", "0.0"]);
    });
    println!("{table}");

    line();

    // Margin State
    println!("Margin State:");
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
        .set_header(vec![
            "Type",
            "Account Value",
            "Total Margin",
            "Total Positions",
            "Total Value",
        ])
        .add_row(vec![
            "Isolated",
            &state.margin_summary.account_value,
            &state.margin_summary.total_margin_used,
            &state.margin_summary.total_ntl_pos,
            &state.margin_summary.total_raw_usd,
        ])
        .add_row(vec![
            "Cross",
            &state.cross_margin_summary.account_value,
            &state.cross_margin_summary.total_margin_used,
            &state.cross_margin_summary.total_ntl_pos,
            &state.cross_margin_summary.total_raw_usd,
        ]);
    println!("{table}");

    // by default, user positions are emitted
    if all_details {
        if state.asset_positions.len() > 0 {
            draw_user_positions_table(state.asset_positions);
        } else {
            println!("No holding positions")
        }
    }
}
