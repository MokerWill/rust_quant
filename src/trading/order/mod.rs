use std::cmp::PartialEq;
use anyhow::anyhow;
use log::error;
use tracing::{info, info_span};
use crate::trading::okx;
use crate::trading::okx::account::{Account, Position, TradingNumResponseData};
use crate::trading::okx::trade;
use crate::trading::okx::trade::{AttachAlgoOrd, CloseOrderRequest, OkxTrade, OrderRequest, OrderResponse, OrderResponseData, OrdType, PosSide, Side, TdMode, TpOrdKind};
use crate::trading::strategy::ut_boot_strategy::SignalResult;

//下单现货
pub async fn place_order_spot(inst_id: &str, side: Side, px: f64) -> anyhow::Result<Vec<OrderResponseData>> {

    //todo 获取当前可以开仓的数量
    let sz = 1;
    //todo 设置止盈止损
    let px = 3000.00;

    let order_params = OrderRequest {
        inst_id: inst_id.to_string(),
        td_mode: TdMode::CASH.to_string(),
        side: side.to_string(),
        ord_type: OrdType::LIMIT.to_string(),
        sz: sz.to_string(),
        px: Option::from(px.to_string()),
        reduce_only: Some(false),
        stp_mode: Some("cancel_maker".to_string()),
        attach_algo_ords: Some(vec![
            AttachAlgoOrd {
                attach_algo_cl_ord_id: None,
                tp_trigger_px: Some("3500".to_string()),
                tp_ord_px: Some("-1".to_string()),
                tp_ord_kind: Some(TpOrdKind::CONDITION.to_string()),
                sl_trigger_px: Some("2200".to_string()),
                sl_ord_px: Some("-1".to_string()),
                tp_trigger_px_type: Some("last".to_string()),
                sl_trigger_px_type: Some("last".to_string()),
                sz: None,
                amend_px_on_trigger_type: Some(0),
            }
        ]),

        ban_amend: None,
        tgt_ccy: None,
        pos_side: None,
        ccy: None,
        cl_ord_id: None,
        tag: None,
        px_usd: None,
        px_vol: None,
        quick_mgn_type: None,
        stp_id: None,
    };
    //下单
    let result = trade::OkxTrade::new().order(order_params).await;

    // okx_response: {"code":"1","data":[{"clOrdId":"","ordId":"","sCode":"51094","sMsg":"You can't place TP limit orders in spot, margin, or options trading.","tag":"","ts":"1718339551210"}],"inTime":"1718339551209444","msg":"All operations failed","outTime":"1718339551210787"}
    // okx_response: {"code":"0","data":[{"clOrdId":"","ordId":"1538100941143183360","sCode":"0","sMsg":"Order placed","tag":"","ts":"1718341380112"}],"inTime":"1718341380111025","msg":"","outTime":"1718341380112306"}

    info!("Order result: {:#?}", result);
    result
}

pub struct OrderSignal {
    pub inst_id: String,
    pub should_sell: bool,
    pub price: f64,
}

pub async fn close_position(position_list: &Vec<Position>, inst_id: &str, pos_side: PosSide) -> anyhow::Result<bool> {
    let already_have_position = position_list.len() > 0;
    //是否已经有反向仓位
    let mut have_anthor_position = false;
    if already_have_position {
        // let position = position_list.get(0).unwrap();
        for position in position_list {
            if position.inst_id == inst_id {
                if position.pos_side == pos_side.to_string() {
                    let params = CloseOrderRequest {
                        inst_id: inst_id.to_string(),
                        pos_side: Option::from(pos_side.to_string()),
                        mgn_mode: TdMode::ISOLATED.to_string(),
                        ccy: None,
                        auto_cxl: Some(true),
                        cl_ord_id: None,
                        tag: None,
                    };
                    let res = OkxTrade::new().close_position(params).await?;
                    info!("close  order position result {:?}", res);
                } else {
                    have_anthor_position = true
                }
            } else {
                error!("close order position not match ");
            }
        }
    }
    Ok(have_anthor_position)
}

pub fn get_place_order_num(avalid_num: &TradingNumResponseData, price: f64, pos_side: PosSide) -> String {
    let size = match pos_side {
        PosSide::LONG => {
            format!("{}", (avalid_num.avail_buy.parse::<f64>().unwrap() / price * 100.00).floor())
        }
        PosSide::SHORT => {
            format!("{}", (avalid_num.avail_sell.parse::<f64>().unwrap() / price * 100.00).floor())
        }
    };
    size.to_string()
}

pub async fn deal(inst_id: &str, time: &str, signal: SignalResult) -> anyhow::Result<()> {
    if signal.should_buy || signal.should_sell {
        //获取当前仓位状态
        let position_list = Account::new().get_account_positions(Some("SWAP"), Some(inst_id), None).await?;

        //获取可用账户可用数量
        let max_avail_size = Account::get_max_avail_size(inst_id, TdMode::ISOLATED).await?;
        info!("max_avail_size: {:?}", max_avail_size);

        if signal.should_buy {
            // 1 如果有空单单则平掉全部空单
            close_position(&position_list, inst_id, PosSide::SHORT).await?;
            //2 获取当前交易产品的仓位,判断当前是否已经持有空单仓位
            let size = get_place_order_num(&max_avail_size, signal.price, PosSide::LONG);
            info!("get allot p size: {:?}", size);
            order_swap(inst_id, Side::BUY, PosSide::LONG, signal.price, size).await?;
        }
        if signal.should_sell {
            // 1 如果有d多单则平掉全部多单
            let already_have_same_position = close_position(&position_list, inst_id, PosSide::LONG).await?;
            //2 获取当前交易产品的仓位,判断当前是否已经持有空单仓位
            let size = get_place_order_num(&max_avail_size, signal.price, PosSide::SHORT);
            info!("get allot p size: {:?}", size);
            // 设置空仓与止盈止损,下单
            order_swap(inst_id, Side::SELL, PosSide::SHORT, signal.price, size).await?;
        }
    }
    Ok(())
}


impl AttachAlgoOrd {
    pub fn new(tp_trigger_px: String, tp_ord_px: String, sl_trigger_px: String, sl_ord_px: String, sz: String) -> Self {
        Self {
            attach_algo_cl_ord_id: None,
            tp_trigger_px: Some(tp_trigger_px),
            tp_ord_px: Some(tp_ord_px),
            tp_ord_kind: Some(TpOrdKind::CONDITION.to_string()),
            sl_trigger_px: Some(sl_trigger_px),
            sl_ord_px: Some(sl_ord_px),
            tp_trigger_px_type: Some("last".to_string()),
            sl_trigger_px_type: Some("last".to_string()),
            sz: Some(sz),
            amend_px_on_trigger_type: Some(1),
        }
    }
}


fn main() {}


pub fn generate_fibonacci_take_profit_orders(
    entry_price: f64,
    fib_levels: &[f64],
    stop_loss_price: f64,
    size: &str,
    side: &Side,
) -> Vec<AttachAlgoOrd> {
    let mut orders = Vec::new();

    //止盈
    for level in fib_levels {
        let tp_trigger_px: f64 = match side {
            Side::SELL => {
                entry_price * (1.0 - level)
            }
            Side::BUY => {
                entry_price * (1.0 + level)
            }
        };

        // fn set_to_multiple(value: &mut i32, multiple_of: i32) {
        // *value = (*value / multiple_of) * multiple_of;
        // }

        // let order_size = (size.parse::<f64>().unwrap() * (level / fib_levels.iter().sum::<f64>())).ceil();
        // let order_size_str = format!("{:.2}", order_size);

        // let tp_ord_px = tp_trigger_px - 100.0; // 根据你的需求调整价格
        let order = AttachAlgoOrd::new(
            format!("{:.2}", tp_trigger_px),
            format!("{:.2}", -1), // 根据你的需求调整价格
            format!("{:.2}", stop_loss_price),
            format!("{:.2}", -1), // 根据你的需求调整价格
            size.to_string(),
        );
        orders.push(order);
    }
    orders
}

//下单合约
pub async fn order_swap(inst_id: &str, side: Side, pos_side: PosSide, entry_price: f64, size: String) -> anyhow::Result<Vec<OrderResponseData>> {

    //止盈
    let stop_loss_price: f64 = match side {
        Side::SELL => {
            entry_price * (1.0 + 0.02)
        }
        Side::BUY => {
            entry_price * (1.0 - 0.02)
        }
    };

    // let stop_loss_price = entry_price * (1.0 - 0.1);
    let fib_levels = [0.0236];
    // let size = "1".to_string();

    let attach_algo_ords = generate_fibonacci_take_profit_orders(entry_price, &fib_levels, stop_loss_price, &size, &side);
    println!("attach_algo_ords{:#?}", attach_algo_ords);

    let order_params = OrderRequest {
        inst_id: inst_id.to_string(),
        td_mode: TdMode::ISOLATED.to_string(),
        ccy: None,
        cl_ord_id: None,
        tag: None,
        side: side.to_string(),
        pos_side: Option::from(pos_side.to_string()),
        // pos_side: None,
        ord_type: OrdType::MARKET.to_string(),
        sz: size,
        px: None,
        // px: Some("30000".to_string()),
        px_usd: None,
        px_vol: None,
        reduce_only: Some(false),
        tgt_ccy: None,
        ban_amend: Some(false),
        quick_mgn_type: None,
        stp_id: None,
        stp_mode: Some("cancel_maker".to_string()),
        attach_algo_ords: Some(attach_algo_ords),
    };
    //下单
    let result = trade::OkxTrade::new().order(order_params).await;
    info!("Order result: {:#?}", result);
    result
}