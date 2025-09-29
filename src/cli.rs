use anyhow::Result;
use clap::builder::ArgPredicate;
use clap::{Parser, Subcommand, ValueEnum};

use hyperliquid_rust_sdk::{
    ClientCancelRequest, ClientLimit, ClientOrder, ClientOrderRequest, ClientTrigger,
    ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus, InfoClient,
};
use hyperliquid_toolset::HyperLiquidConfig;
use hyperliquid_toolset::tui::LivePanel;
use hyperliquid_toolset::types::PriceIndex;
use hyperliquid_toolset::ui;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Parser)]
#[command(author, version, about="A cmd hyperliquid toolset for traders", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// 限价订单类型
/// - Alo: Add Liquidity Only
/// - Ioc: Immediate or cancel
/// - Gtc: Good till cancel
#[derive(Debug, Clone, ValueEnum, Default, PartialEq, Eq)]
#[clap(rename_all = "verbatim")]
enum LimitType {
    /// 订单有效直到取消或成交，适合长期挂单，可能为 Maker 或 Taker
    #[default]
    Gtc,
    /// 立即成交或取消，适合快速执行，通常为 Taker
    Ioc,
    /// 仅添加流动性，确保 Maker 费率，适合做市和成本优化
    Alo,
}

/// 订单类型
#[derive(Debug, Clone, Subcommand)]
enum OrderType {
    /// 订单类型：限价订单
    Limit {
        #[arg(short, long, requires = "price")]
        limit: LimitType,
    },

    /// 订单类型：触发器订单
    Trigger {
        /// Trigger Price, 触发价格
        #[arg(short, long)]
        trigger_price: String,
        /// 以限定价执行，否则按市场价执行
        #[arg(short, long, action, requires_if(ArgPredicate::IsPresent, "price"))]
        limit_price: bool,
        /// 触发类型： 止损/止赢
        #[arg(long)]
        tpsl: TriggerType,
    },
}

/// 订单方向
#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
#[clap(rename_all = "lower")]
enum OrderSide {
    /// 订单方向: buy
    Buy,
    /// 订单方向: sell
    Sell,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
#[clap(rename_all = "lower")]
enum TriggerType {
    /// 触发类型: Take Profit 止赢
    Tp,
    /// 触发类型: Stop Loss 止损
    Sl,
}

#[derive(Debug, Parser)]
struct OrderRequest {
    /// 交易对
    #[arg(short, long)]
    asset: String,

    /// 订单类型
    #[command(subcommand)]
    order: OrderType,

    /// 订单方向
    #[arg(short, long)]
    side: OrderSide,

    /// 只减仓
    #[arg(short, long, action, default_value = "false")]
    reduce_only: bool,

    /// 价格
    #[arg(short, long, global = true)]
    price: Option<f64>,

    /// 数量
    #[arg(short, long)]
    quantity: f64,

    /// cloid
    #[arg(short, long)]
    cloid: Option<String>,
}

#[derive(Debug, Parser)]
struct CancelRequest {
    /// 交易对
    #[arg(short, long)]
    asset: String,
    /// 订单ID
    #[arg(short, long)]
    order_id: u64,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// 获取所有标的价格
    AllMids {
        /// 是否动态刷新价格
        #[arg(short, long, action)]
        live: bool,
        /// 刷新时间间隔,单位秒
        #[arg(short, long, default_value = "5")]
        interval: Option<u64>,
    },
    /// 查看账户余额
    Balance {
        /// 是否显示所有资产
        #[arg(short, long)]
        all: bool,
    },
    /// 查询活跃订单
    Orders,
    /// 查看仓位
    Positions,
    /// 下单
    Order(OrderRequest),
    /// 取消订单
    Cancel(CancelRequest),
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();
    let hl_config = HyperLiquidConfig::new();

    let info_client = InfoClient::new(None, None).await?;
    let wallet = hl_config.wallet()?;
    let exchange_client = ExchangeClient::new(None, wallet, None, None, None).await?;

    match cli.command {
        Commands::Balance { all } => {
            let (state, balance) = tokio::join! {
                info_client.user_state(hl_config.account_address),
                info_client.user_token_balances(hl_config.account_address),
            };
            let state = state?;
            let balance = balance?;
            ui::draw_balance_table(state, balance.balances, all);
        }
        Commands::Orders => {
            let orders = info_client.open_orders(hl_config.account_address).await?;
            ui::draw_orders_table(orders);
        }
        Commands::AllMids { live, interval } => {
            if live {
                let interval = interval.unwrap_or(5);
                let mut ui = LivePanel::with_updater(|| async {
                    let _ = sleep(Duration::from_secs(interval)).await;
                    let result = info_client.all_mids().await;
                    match result {
                        Ok(response) => {
                            let mids: Vec<PriceIndex> = response
                                .iter()
                                .filter(|(k, _)| !k.starts_with("@"))
                                .map(|(k, v)| PriceIndex {
                                    asset: k.clone(),
                                    price: v.clone().parse::<f64>().unwrap_or(0.0),
                                })
                                .collect();
                            mids
                        }
                        Err(error) => {
                            println!("  🔴 Error: {}", error);
                            vec![]
                        }
                    }
                });
                let _ = ui.run_tui().await;
            } else {
                let result = info_client.all_mids().await;
                match result {
                    Ok(response) => {
                        let mids: Vec<PriceIndex> = response
                            .iter()
                            .map(|(k, v)| PriceIndex {
                                asset: k.clone(),
                                price: v.clone().parse::<f64>().unwrap_or(0.0),
                            })
                            .collect();
                        ui::draw_all_mids_table(&mids);
                    }
                    Err(error) => {
                        println!("  🔴 Error: {}", error);
                    }
                }
            }
        }
        Commands::Positions => {
            let result = info_client.user_state(hl_config.account_address).await;
            match result {
                Ok(response) => {
                    let positions = response.asset_positions;
                    ui::draw_user_positions_table(positions);
                }
                Err(error) => {
                    println!("  🔴 Error: {}", error);
                }
            }
        }
        Commands::Order(OrderRequest {
            asset,
            side,
            reduce_only,
            order,
            price,
            quantity,
            cloid,
        }) => {
            let order_type = match order {
                OrderType::Limit { ref limit } => ClientOrder::Limit(ClientLimit {
                    tif: format!("{:?}", limit),
                }),
                OrderType::Trigger {
                    ref trigger_price,
                    limit_price,
                    ref tpsl,
                } => ClientOrder::Trigger(ClientTrigger {
                    is_market: !limit_price,
                    trigger_px: trigger_price.parse::<f64>().unwrap(),
                    tpsl: format!("{:?}", tpsl).to_lowercase(),
                }),
            };

            let price = if let OrderType::Trigger {
                ref trigger_price,
                limit_price,
                ..
            } = order
                && !limit_price
            {
                // market price actually, so we can miss the price parameter
                trigger_price.parse::<f64>().unwrap()
            } else {
                price.unwrap()
            };

            let cloid = if let Some(uuid) = cloid {
                Some(uuid::Uuid::from_str(&uuid).unwrap())
            } else {
                None
            };

            let order_request = ClientOrderRequest {
                asset,
                is_buy: side == OrderSide::Buy,
                reduce_only: reduce_only,
                limit_px: price,
                sz: quantity,
                cloid,
                order_type,
            };

            let order_result = exchange_client.order(order_request, None).await;
            match order_result {
                Ok(response) => match response {
                    ExchangeResponseStatus::Ok(status) => {
                        println!("Operation result: ");
                        ui::line();
                        println!("  Type: {}", status.response_type);
                        if let Some(data) = status.data {
                            for status in data.statuses {
                                match status {
                                    ExchangeDataStatus::Success => {
                                        println!("  🟢 Success");
                                    }
                                    ExchangeDataStatus::WaitingForTrigger
                                    | ExchangeDataStatus::WaitingForFill => {
                                        println!("  🟠 Waiting for fill/trigger");
                                    }
                                    ExchangeDataStatus::Filled(filled) => {
                                        println!(
                                            "  🟢 Filled: {} @ {}, Order ID: {}",
                                            filled.total_sz, filled.avg_px, filled.oid,
                                        );
                                    }
                                    ExchangeDataStatus::Resting(data) => {
                                        println!("  🟢 Order ID: {}", data.oid);
                                    }
                                    ExchangeDataStatus::Error(err) => {
                                        println!("  🔴 Failure: {}", err);
                                    }
                                }
                            }
                        }
                    }
                    ExchangeResponseStatus::Err(err) => {
                        println!("  🔴 Error: {}", err);
                    }
                },
                Err(err) => {
                    println!("  🔴 Error: {}", err);
                }
            }
        }
        Commands::Cancel(CancelRequest { asset, order_id }) => {
            let cancel_request = ClientCancelRequest {
                asset,
                oid: order_id,
            };
            match exchange_client.cancel(cancel_request, None).await {
                Ok(response) => match response {
                    ExchangeResponseStatus::Ok(status) => {
                        if status.data.is_some() {
                            match &status.data.unwrap().statuses[0] {
                                ExchangeDataStatus::Success => {
                                    println!("  🟢 Success");
                                }
                                ExchangeDataStatus::Filled(filled) => {
                                    println!(
                                        "  🟢 Success: {}@{} by {}",
                                        filled.oid, filled.avg_px, filled.total_sz,
                                    );
                                }
                                ExchangeDataStatus::WaitingForFill
                                | ExchangeDataStatus::WaitingForTrigger => {
                                    println!("  🟡 Wating for Fill/Trigger..");
                                }
                                ExchangeDataStatus::Resting(reset) => {
                                    println!("  🔄 Resting Order: {}", reset.oid);
                                }
                                ExchangeDataStatus::Error(error) => {
                                    println!("  🔴 Error: {}", error)
                                }
                            }
                        }
                    }
                    ExchangeResponseStatus::Err(error) => {
                        println!("  🔴 Error: {}", error)
                    }
                },
                Err(error) => {
                    println!("  🔴 Error: {}", error)
                }
            };
        }
    }

    Ok(())
}
