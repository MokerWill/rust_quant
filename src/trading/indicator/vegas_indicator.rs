use std::fmt::Display;
use std::sync::Arc;
use serde_json::json;
use ta::{Close, DataItem, High, Low, Next, Volume};
use ta::indicators::ExponentialMovingAverage;
use ta::indicators::{RelativeStrengthIndex, MovingAverageConvergenceDivergence};
use crate::trading::indicator::rsi_rma::Rsi;
use crate::trading::model::market::candles::CandlesEntity;
use crate::trading::strategy::strategy_common::{BackTestResult, SignalResult, TradingStrategyConfig};
use crate::trading::strategy::strategy_common;
use crate::trading::indicator::signal_weight::{SignalWeights, SignalScore, SignalType, SignalCondition};

// 成交量趋势
#[derive(Debug)]
pub struct VolumeTrend {
    pub is_increasing: bool,
    pub is_decreasing: bool,
    pub is_stable: bool, // 是否稳定
    pub volume_ratio: f64,  // 添加 volume_ratio 字段
}

// 成交量信号配置
#[derive(Debug)]
pub struct VolumeSignal {
    pub volume_bar_num: usize,  // 看前10根K线
    pub volume_increase_ratio: f64,  // 放量倍数
    pub volume_decrease_ratio: f64,  // 缩量倍数
    pub is_open: bool,  // 缩量倍数
}


// ema信号配置
#[derive(Debug)]
pub struct EmaSignal {
    pub ema1_length: usize,
    pub ema2_length: usize,
    pub ema3_length: usize,
    pub ema4_length: usize,
    pub ema5_length: usize,
    pub ema_breakthrough_threshold: f64,  // 新增：ema突破价格的阈值
    pub is_open: bool,
}

#[derive(Debug)]
pub struct EmaValue {
    pub ema1_value: f64,
    pub ema2_value: f64,
    pub ema3_value: f64,
    pub ema4_value: f64,
    pub ema5_value: f64,
}

// rsi信号配置
#[derive(Debug)]
pub struct RsiSignal {
    pub rsi_length: usize, // rsi周期
    pub rsi_oversold: f64, // rsi超卖阈值
    pub rsi_overbought: f64, // rsi超买阈值
    pub is_open: bool, // 是否开启
}


impl VolumeTrend {
    pub fn new(is_increasing: bool, is_decreasing: bool, is_stable: bool, volume_ratio: f64) -> Self {
        Self {
            is_increasing,
            is_decreasing,
            is_stable,
            volume_ratio,
        }
    }
}

// ema趋势
#[derive(Debug)]
pub struct EmaTouchTrendSignal {
   pub ema2_with_ema3_ratio: f64,//eam2与eam3的相差幅度
    pub ema3_with_ema4_ratio: f64,//eam2与eam3的相差幅度
    pub ema4_with_ema5_ratio: f64,//eam2与eam3的相差幅度
   pub is_open: bool, //是否开启
}
impl Default for EmaTouchTrendSignal {
    fn default() -> Self {
        Self {
            ema4_with_ema5_ratio: 0.09,
            ema3_with_ema4_ratio: 0.09,
            ema2_with_ema3_ratio: 0.09,
            is_open: false,
        }
    }
}
// ema趋势
#[derive(Debug)]
pub struct EmaTouchTrendValue {
    pub is_uptrend: bool, //是否多头趋势
    pub is_downtrend: bool, //是否空头趋势
    pub is_in_uptrend_touch_ema2: bool, //是否在多头趋势触碰ema2
    pub is_in_uptrend_touch_ema3: bool, //是否在多头趋势触碰ema3
    pub is_in_uptrend_touch_ema2_ema3_nums: usize, //当前多头趋势中触碰ema2和ema3的次数

    pub is_in_uptrend_touch_ema4: bool, //是否在多头趋势触碰ema4
    pub is_in_uptrend_touch_ema5: bool, //是否在多头趋势触碰ema4
    pub is_in_uptrend_touch_ema4_ema5_nums: usize, //当前多头趋势中触碰ema4和ema5的次数

    pub is_in_downtrend_touch_ema2: bool, //是否在空头趋势触碰ema2
    pub is_in_downtrend_touch_ema3: bool, //是否在空头趋势触碰ema3
    pub is_in_downtrend_touch_ema2_ema3_nums: usize, //当前空头趋势触碰ema2和ema3的次数

    pub is_in_downtrend_touch_ema4: bool, //是否在空头趋势触碰ema4
    pub is_in_downtrend_touch_ema5: bool, //是否在空头趋势触碰ema5
    pub is_in_downtrend_touch_ema4_ema5_nums: usize, //当前空头趋势中触碰ema4和ema5的次数

    pub is_long_signal: bool, //是否多头开仓
    pub is_short_signal: bool, //是否空头开仓
}
impl Default for EmaTouchTrendValue {
    fn default() -> Self {
        Self {
            is_uptrend: false,
            is_downtrend: false,
            is_in_uptrend_touch_ema2: false,
            is_in_uptrend_touch_ema3: false,
            is_in_uptrend_touch_ema2_ema3_nums: 0,
            is_in_uptrend_touch_ema4: false,
            is_in_uptrend_touch_ema5: false,
            is_in_uptrend_touch_ema4_ema5_nums: 0,
            is_in_downtrend_touch_ema2: false,
            is_in_downtrend_touch_ema3: false,
            is_in_downtrend_touch_ema2_ema3_nums: 0,
            is_in_downtrend_touch_ema4: false,
            is_in_downtrend_touch_ema5: false,
            is_in_downtrend_touch_ema4_ema5_nums: 0,
            is_long_signal: false,
            is_short_signal: false,
        }
    }
}

// 新增：检查均线交叉
pub struct EmaCross {
    pub is_golden_cross: bool,
    pub is_death_cross: bool,
}

pub struct VegasIndicator {
    pub ema_signal: EmaSignal, // ema信号配置
    pub volume_signal: VolumeSignal,  // 新增：成交量信号配置
    pub ema_touch_trend_signal: EmaTouchTrendSignal, // ema趋势
    pub rsi_signal: RsiSignal,
    pub signal_weights: SignalWeights,  // 新增权重配置
}

impl Display for VegasIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "vegas indicator :ema0:{} ema2:{} ema3:{}", self.ema_signal.ema1_length, self.ema_signal.ema2_length, self.ema_signal.ema3_length)
    }
}
impl VegasIndicator {
    pub fn new(ema1: usize, ema2: usize, ema3: usize, ema4: usize, ema5: usize) -> Self {
        Self {
            ema_touch_trend_signal: EmaTouchTrendSignal::default(),
            ema_signal: EmaSignal {
                ema1_length: ema1,
                ema2_length: ema2,
                ema3_length: ema3,
                ema4_length: ema4,
                ema5_length: ema5,
                ema_breakthrough_threshold: 0.003,
                is_open: true,
            },
            //11
            volume_signal: VolumeSignal {
                volume_bar_num: 10,
                volume_increase_ratio: 2.5,
                volume_decrease_ratio: 0.5,
                is_open: true,
            },
            rsi_signal: RsiSignal {
                rsi_length: 12,  // 默认RSI周期
                rsi_oversold: 25.0,            // 默认25
                rsi_overbought: 75.0,          // 默认75
                is_open: true,
            },
            signal_weights: SignalWeights {
                weights: vec![
                    (SignalType::Breakthrough, 2.0),
                    (SignalType::VolumeTrend, 1.5),
                    (SignalType::Rsi, 1.0),
                    (SignalType::TrendStrength, 1.5),
                    (SignalType::EmaDivergence, 1.8),
                    (SignalType::PriceLevel, 1.2),
                ],
                min_total_weight: 3.0,  // 需要至少3分才触发信号
            },
        }
    }
    pub fn get_min_data_length(&mut self) -> usize {
        800
    }
    pub fn calculate(&mut self, data: &[DataItem]) -> EmaValue {  // 修改返回值
        // 确保数据量足够
        if data.len() < self.ema_signal.ema5_length {  // 使用最长的EMA周期
            return EmaValue {
                ema1_value: 0.0,
                ema2_value: 0.0,
                ema3_value: 0.0,
                ema4_value: 0.0,
                ema5_value: 0.0,
            };
        }

        let mut ema1 = ExponentialMovingAverage::new(self.ema_signal.ema1_length).unwrap();
        let mut ema2 = ExponentialMovingAverage::new(self.ema_signal.ema2_length).unwrap();
        let mut ema3 = ExponentialMovingAverage::new(self.ema_signal.ema3_length).unwrap();
        let mut ema4 = ExponentialMovingAverage::new(self.ema_signal.ema4_length).unwrap();  // 新增
        let mut ema5 = ExponentialMovingAverage::new(self.ema_signal.ema5_length).unwrap();  // 新增

        let mut ema1_value = 0.0;
        let mut ema2_value = 0.0;
        let mut ema3_value = 0.0;
        let mut ema4_value = 0.0;  // 新增
        let mut ema5_value = 0.0;  // 新增

        // 计算初始 SMA
        let sma1: f64 = data[0..self.ema_signal.ema1_length].iter().map(|x| x.close()).sum::<f64>() / self.ema_signal.ema1_length as f64;
        let sma2: f64 = data[0..self.ema_signal.ema2_length].iter().map(|x| x.close()).sum::<f64>() / self.ema_signal.ema2_length as f64;
        let sma3: f64 = data[0..self.ema_signal.ema3_length].iter().map(|x| x.close()).sum::<f64>() / self.ema_signal.ema3_length as f64;
        let sma4: f64 = data[0..self.ema_signal.ema4_length].iter().map(|x| x.close()).sum::<f64>() / self.ema_signal.ema4_length as f64;  // 新增
        let sma5: f64 = data[0..self.ema_signal.ema5_length].iter().map(|x| x.close()).sum::<f64>() / self.ema_signal.ema5_length as f64;  // 新增

        // 设置初始值
        ema1_value = sma1;
        ema2_value = sma2;
        ema3_value = sma3;
        ema4_value = sma4;  // 新增
        ema5_value = sma5;  // 新增

        // 计算EMA值
        for i in 0..data.len() {
            let close = data[i].close();

            if i >= self.ema_signal.ema1_length { ema1_value = ema1.next(close); }
            if i >= self.ema_signal.ema2_length { ema2_value = ema2.next(close); }
            if i >= self.ema_signal.ema3_length { ema3_value = ema3.next(close); }
            if i >= self.ema_signal.ema4_length { ema4_value = ema4.next(close); }  // 新增
            if i >= self.ema_signal.ema5_length { ema5_value = ema5.next(close); }  // 新增
        }
        let ema_value = EmaValue {
            ema1_value,
            ema2_value,
            ema3_value,
            ema4_value,
            ema5_value,
        };
        ema_value
    }

    pub fn convert_to_data_items(&self, prices: &Vec<CandlesEntity>) -> Vec<DataItem> {
        prices.iter().map(|candle| {
            DataItem::builder().open(candle.o.parse().unwrap()).high(candle.h.parse().unwrap()).low(candle.l.parse().unwrap()).close(candle.c.parse().unwrap()).volume(candle.vol.parse().unwrap()).build().unwrap()
        }).collect()
    }
//22
    pub fn get_trade_signal(&mut self, data: &[CandlesEntity], weights: &SignalWeights) -> SignalResult {
        let mut signal_result = SignalResult {
            should_buy: false,
            should_sell: false,
            price: 0.0,
            ts: 0,
            single_detail: None,
        };

        // 转换数据
        let data_items = self.convert_to_data_items(&data.to_vec());
        if data_items.len() < self.ema_signal.ema5_length + 10 {
            println!("数据长度不足: {} < {}", data_items.len(), self.ema_signal.ema5_length + 10);
            return signal_result;
        }

        // 计算ema
        let ema_value = self.calculate(&data_items);

        let current_price = data.last().unwrap().c.parse::<f64>().unwrap();
        let lower_price = data.last().unwrap().l.parse::<f64>().unwrap();

//33
        // 2. ema2 与价格关系判断
        let (price_above, price_below) = self.check_breakthrough_conditions(&data_items, ema_value.ema2_value);

        let mut conditions = vec![
            (SignalType::Breakthrough, SignalCondition::PriceBreakout {
                price_above,
                price_below,
            }),
        ];

        //新增函数 判断ema 是否空头排列，或者多头排列或者多头排列
    if self.ema_touch_trend_signal.is_open {
        let ema_trend = self.calculate_ema_trend(&data_items, ema_value);
        println!("ema趋势: 多头={}, 空头={}", ema_trend.is_uptrend, ema_trend.is_downtrend);
        if ema_trend.is_long_signal || ema_trend.is_short_signal {
            conditions.push((SignalType::EmaTrend, SignalCondition::EmaTrend {
                is_long_signal: ema_trend.is_long_signal,
                is_short_signal: ema_trend.is_short_signal,
            }));
        }
    }

            //成交量
            if self.volume_signal.is_open {
                let volume_trend = self.check_volume_trend(&data_items);
                println!("成交量趋势: 增加={}, 减少={}, 稳定={}", volume_trend.is_increasing, volume_trend.is_decreasing, volume_trend.is_stable);
                conditions.push((SignalType::VolumeTrend, SignalCondition::Volume { is_increasing: volume_trend.is_increasing, ratio: volume_trend.volume_ratio }))
            }

            // 计算RSI
            if self.rsi_signal.is_open {
                let mut rsi = Rsi::new(self.rsi_signal.rsi_length);
                let rsi_values: Vec<f64> = data_items.iter().map(|item| rsi.next(item.close())).collect();
                let current_rsi = *rsi_values.last().unwrap();
                println!("RSI: {:.2}", current_rsi);
                conditions.push((SignalType::Rsi, SignalCondition::RsiLevel {
                    current: current_rsi,
                    oversold: self.rsi_signal.rsi_oversold,
                    overbought: self.rsi_signal.rsi_overbought,
                }));
            }


            // let trend_strength = self.calculate_trend_strength(&data_items);
            // println!("时间：{:?} 价格: {:.3}, EMA2: {:.4}, EMA3: {:.4}  ema4: {:.4}  ema5: {:.4}", crate::time_util::mill_time_to_datetime_shanghai(data.last().unwrap().ts), current_price, ema2_value, ema3_value, ema4_value, ema5_value);
            // println!("趋势: 上升={}, 下降={}", ema_trend.is_uptrend, ema_trend.is_downtrend);
            // println!("趋势强度: {:.3}", trend_strength);


            // 检查突破的持续性
            let breakthrough_confirmed = self.check_breakthrough_confirmation(&data_items, price_above);


            // (SignalType::Rsi, SignalCondition::RsiLevel {
            //     current: current_rsi,
            //     oversold: self.rsi_oversold,
            //     overbought: self.rsi_overbought,
            // }),
            // (SignalType::TrendStrength, SignalCondition::Trend {
            //     strength: trend_strength,
            //     threshold: 0.6,
            // }),
            //(SignalType::EmaDivergence, SignalCondition::EmaStatus {
            //     is_diverging: ema2_value > ema3_value * 1.02,
            // }),
            println!("condition: {:?}", conditions);
            let score = weights.calculate_score(conditions.clone());
            if weights.is_signal_valid(&score) {
                signal_result.should_buy = true;
                signal_result.single_detail = Some(json!(conditions).to_string());
            }

            signal_result.price = current_price;
            signal_result.ts = data.last().unwrap().ts;

            if signal_result.should_buy || signal_result.should_sell {
                println!("产生信号: {}", signal_result.single_detail.as_ref().unwrap());
            }

            signal_result

    }

        // 辅助方法：检查成交量是否显著增加
        // fn check_volume_increase(&self, data: &[DataItem]) -> bool {
        //     if data.len() < 5 { return false; }
        //
        //     let current_volume = data.last().unwrap().volume();  // 使用真实成交量
        //     let avg_volume: f64 = data[data.len() - 6..data.len() - 1].iter().map(|x| x.volume())  // 使用真实成交量.sum::<f64>() / 5.0;
        //
        //     // println!("成交量检查 - 当前: {}, 平均: {}, 倍数: {}", current_volume, avg_volume, current_volume / avg_volume);
        //     current_volume > avg_volume * self.volume_signal.volume_increase_ratio // 倍数大于1.5
        // }
        // 辅助方法：计算EMA趋势
     fn calculate_ema_trend(&mut self, data: &[DataItem], ema_value: EmaValue) -> EmaTouchTrendValue {
            //判断ema 是否空头排列，或者多头排列或者多头排列
            let mut ema_touch_trend_value = EmaTouchTrendValue::default();

            if ema_value.ema1_value > ema_value.ema2_value && ema_value.ema2_value > ema_value.ema3_value && ema_value.ema3_value > ema_value.ema4_value && ema_value.ema4_value > ema_value.ema5_value {
                //判断是否多头趋势
                ema_touch_trend_value.is_uptrend = true;
                //当价格第到达 mea2 ema 3位置时候
                if data.last().unwrap().close() <= ema_value.ema2_value || data.last().unwrap().close() <= ema_value.ema3_value {
                    ema_touch_trend_value.is_in_uptrend_touch_ema2_ema3_nums += 1;
                    if data.last().unwrap().close() <= ema_value.ema2_value {
                        ema_touch_trend_value.is_in_uptrend_touch_ema2 = true;
                    } else {
                        ema_touch_trend_value.is_in_uptrend_touch_ema3 = true;
                    }
                    //且大幅>ema4的时候，第一次做多
                    let bool2 = data.last().unwrap().close() > ema_value.ema4_value * self.ema_touch_trend_signal.ema3_with_ema4_ratio;
                    //且前一根k线的价格>于ema2
                    let bool3 = data.get(data.len() - 1).unwrap().close() > ema_value.ema2_value;
                    if bool2 && bool3 {
                        ema_touch_trend_value.is_long_signal = true;
                    }
                }
            } else if ema_value.ema1_value < ema_value.ema2_value && ema_value.ema2_value < ema_value.ema3_value && ema_value.ema3_value < ema_value.ema4_value && ema_value.ema4_value < ema_value.ema5_value {
                //判断是否空头趋势
                ema_touch_trend_value.is_downtrend = true;
                //当价格第一次到达 mea2 ema 3位置时候
                if data.last().unwrap().close() >= ema_value.ema2_value && data.last().unwrap().close() < ema_value.ema3_value {
                    ema_touch_trend_value.is_in_downtrend_touch_ema2_ema3_nums += 1;
                    if data.last().unwrap().close() >= ema_value.ema2_value {
                        ema_touch_trend_value.is_in_downtrend_touch_ema2 = true;
                    } else {
                        ema_touch_trend_value.is_in_downtrend_touch_ema3 = true;
                    }
                }
                //且大幅小于ema4的时候，第一次做空
                let bool2 = data.last().unwrap().close() < ema_value.ema4_value * self.ema_touch_trend_signal.ema3_with_ema4_ratio;
                //且前一根k线的价格小于ema2
                let bool3 = data.get(data.len() - 1).unwrap().close() < ema_value.ema2_value;
                if bool2 && bool3 {
                    ema_touch_trend_value.is_short_signal = true;
                }
            }

            if ema_value.ema2_value > ema_value.ema3_value && ema_value.ema3_value > ema_value.ema4_value && ema_value.ema4_value > ema_value.ema5_value {
                ema_touch_trend_value.is_uptrend = true;
                //当价格到达ema4 ,ema5位置时候
                if data.last().unwrap().close() <= ema_value.ema4_value || data.last().unwrap().close() <= ema_value.ema5_value {
                    ema_touch_trend_value.is_in_uptrend_touch_ema4_ema5_nums += 1;
                    if data.last().unwrap().close() <= ema_value.ema4_value {
                        ema_touch_trend_value.is_in_uptrend_touch_ema4 = true;
                    } else {
                        ema_touch_trend_value.is_in_uptrend_touch_ema5 = true;
                    }
                    //且大幅>ema4的时候，第一次做多
                    let bool2 = data.last().unwrap().close() > ema_value.ema4_value * self.ema_touch_trend_signal.ema3_with_ema4_ratio;
                    //且前一根k线的价格>于ema2
                    let bool3 = data.get(data.len() - 1).unwrap().close() > ema_value.ema4_value;
                    if bool2 && bool3 {
                        ema_touch_trend_value.is_long_signal = true;
                    }
                }
            } else if ema_value.ema2_value < ema_value.ema3_value && ema_value.ema3_value < ema_value.ema4_value && ema_value.ema4_value < ema_value.ema5_value {
                ema_touch_trend_value.is_downtrend = true;
                //当价格到达ema4 ,ema5位置时候
                if data.last().unwrap().close() >= ema_value.ema4_value && data.last().unwrap().close() < ema_value.ema5_value {
                    ema_touch_trend_value.is_in_downtrend_touch_ema4_ema5_nums += 1;
                    if data.last().unwrap().close() >= ema_value.ema4_value {
                        ema_touch_trend_value.is_in_downtrend_touch_ema4 = true;
                    } else {
                        ema_touch_trend_value.is_in_downtrend_touch_ema5 = true;
                    }
                    //且大幅<ema4的时候，第一次做空
                    let bool2 = data.last().unwrap().close() < ema_value.ema4_value * self.ema_touch_trend_signal.ema3_with_ema4_ratio;
                    //且前一根k线的价格<于ema2
                    let bool3 = data.get(data.len() - 1).unwrap().close() < ema_value.ema4_value;
                    if bool2 && bool3 {
                        ema_touch_trend_value.is_short_signal = true;
                    }
                }
            }

           ema_touch_trend_value

        }

        // 检查突破信号
    fn check_breakout_signals(&self, price: f64, ema2: f64, ema3: f64, trend: &EmaTouchTrendValue, volume_increase: bool) -> bool {
            let price_above_ema2 = price > ema2;
            let price_below_ema3 = price < ema3;
            // 简化判断条件
            price_above_ema2 || price_below_ema3
        }


        /// Runs the backtest asynchronously.
        pub fn run_test(
            &mut self,
            candles: &Vec<CandlesEntity>,
            fib_levels: &Vec<f64>,
            strategy_config: TradingStrategyConfig,
            is_fibonacci_profit: bool,
            is_open_long: bool,
            is_open_short: bool,
            is_judge_trade_time: bool,
        ) -> BackTestResult {
            let min_length = self.get_min_data_length();
            strategy_common::run_test(
                |candles| self.get_trade_signal(candles, &SignalWeights::default()),
                candles,
                fib_levels,
                strategy_config,
                min_length,
                is_fibonacci_profit,
                is_open_long,
                is_open_short,
            )
        }

        // fn check_ema_cross(&mut self, data: &[DataItem]) -> EmaCross {
        //     let recent_data = &data[data.len() - 3..];
        //     let mut ema_values = Vec::new();
        //
        //     for window in recent_data.windows(2) {
        //         let ema_values = self.calculate(window);
        //         let prev_diff = ema_values.ema2_value - ema_values.ema3_value;
        //         let curr_diff = ema_values.ema2_value - ema_values.ema3_value;
        //         // 金叉：EMA2从下穿越EMA3
        //         let is_golden = prev_diff < 0.0 && curr_diff > 0.0;
        //         // 死叉：EMA2从上穿越EMA3
        //         let is_death = prev_diff > 0.0 && curr_diff < 0.0;
        //         ema_values.push((is_golden, is_death));
        //     }
        //
        //     EmaCross {
        //         is_golden_cross: ema_values.iter().any(|&(g, _)| g),
        //         is_death_cross: ema_values.iter().any(|&(_, d)| d),
        //     }
        // }

        // 修改：计算趋势强度，使用EMA12的短期趋势
        fn calculate_trend_strength(&mut self, data: &[DataItem]) -> f64 {
            const TREND_LOOKBACK: usize = 5;  // 看最近5根K线的趋势

            if data.len() < TREND_LOOKBACK + self.ema_signal.ema1_length {
                return 0.5;
            }

            // 计算包含足够历史数据的EMA序列
            let calc_range = &data[data.len() - TREND_LOOKBACK - self.ema_signal.ema1_length..];
            let mut ema1 = ExponentialMovingAverage::new(self.ema_signal.ema1_length).unwrap();
            let mut ema1_values = Vec::new();

            // 先计算EMA初始值
            let sma1: f64 = calc_range[0..self.ema_signal.ema1_length].iter().map(|x| x.close()).sum::<f64>() / self.ema_signal.ema1_length as f64;
            ema1_values.push(sma1);

            // 连续计算EMA值
            for i in self.ema_signal.ema1_length..calc_range.len() {
                let ema_value = ema1.next(calc_range[i].close());
                ema1_values.push(ema_value);
            }

            // 只取最后TREND_LOOKBACK个值计算趋势
            let trend_values = &ema1_values[ema1_values.len() - TREND_LOOKBACK..];

            // 计算EMA12的角度（斜率）
            let ema1_angle = (trend_values.last().unwrap() - trend_values.first().unwrap()) / trend_values.first().unwrap();

            // 计算当前价格与EMA12的距离
            let current_price = data.last().unwrap().close();
            let price_distance = (current_price - trend_values.last().unwrap()).abs() / trend_values.last().unwrap();

            println!("趋势角度分析 - EMA12角度: {:.4}, 价格距离: {:.4}", ema1_angle, price_distance);
            println!("EMA12序列: {:?}", trend_values);

            // 综合评分 (0.0-1.0)
            let strength = (ema1_angle.abs() * 0.8 + (1.0 - price_distance) * 0.2).max(0.0).min(1.0);

            strength
        }

        // 新增：检查突破确认
     fn check_breakthrough_confirmation(&self, data: &[DataItem], is_upward: bool) -> bool {
            // 实现突破确认逻辑
            // 可以检查:
            // 1. 突破后的持续性
            // 2. 回测支撑/阻力的表现
            // 3. 成交量配合
            true // 临时返回值
        }

        // 新增：计算动态回调幅度
     fn calculate_dynamic_pullback_threshold(&self, _data: &[DataItem]) -> f64 {
            // 实现动态回调幅度计算逻辑
            // 可以考虑:
            // 1. 价格波动性
            // 2. 均线角度
            // 3. 成交量变化
            // 返回回调幅度
            0.005 // 临时返回值
        }

        // 修改成交量趋势判断
    fn check_volume_trend(&self, data: &[DataItem]) -> VolumeTrend {
            if data.len() < self.volume_signal.volume_bar_num + 1 {
                return VolumeTrend {
                    is_increasing: false,
                    is_decreasing: false,
                    is_stable: true,
                    volume_ratio: 0.0,
                };
            }

            let current_volume = data.last().unwrap().volume();

            // 计算前N根K线的平均成交量
            let prev_volumes: Vec<f64> = data[data.len() - self.volume_signal.volume_bar_num - 1..data.len() - 1].iter().map(|x| x.volume()).collect();
            let avg_volume = prev_volumes.iter().sum::<f64>() / prev_volumes.len() as f64;

            // 计算当前成交量与平均值的比值
            let volume_ratio = current_volume / avg_volume;

            println!("成交量分析 - 当前成交量: {:.2}, 平均成交量: {:.2}, 比值: {:.2}",
                     current_volume, avg_volume, volume_ratio);

            VolumeTrend {
                is_increasing: volume_ratio > self.volume_signal.volume_increase_ratio,  // 放量
                is_decreasing: volume_ratio < self.volume_signal.volume_decrease_ratio,  // 缩量
                is_stable: volume_ratio >= self.volume_signal.volume_decrease_ratio && volume_ratio <= self.volume_signal.volume_increase_ratio,  // 稳定
                volume_ratio,
            }
        }

        // 优化：检查关键价位卖出信号
    fn check_key_price_level_sell(&self, current_price: f64, volume_trend: &VolumeTrend) -> Option<String> {
            // 定义价位级别和对应的提前预警距离
            const PRICE_LEVELS: [(f64, f64, f64, &str); 8] = [
                // (价位区间, 提前预警百分比, 建议回撤百分比, 级别描述)
                (10000.0, 0.02, 0.015, "万元"),     // 万元级别
                (1000.0, 0.015, 0.01, "千元"),      // 千元级别
                (100.0, 0.01, 0.008, "百元"),       // 百元级别
                (10.0, 0.008, 0.005, "十元"),       // 十元级别
                (1.0, 0.005, 0.003, "元"),          // 1元级别
                (0.1, 0.003, 0.002, "角"),          // 0.1元级别
                (0.01, 0.002, 0.001, "分"),         // 0.01元级别
                (0.001, 0.001, 0.0005, "厘")        // 0.001元级别
            ];

            // 修改：从大到小遍历找到第一个小于等于当前价格的级别
            let (interval, alert_percent, pullback_percent, level_name) = PRICE_LEVELS.iter().find(|&&(level, _, _, _)| current_price >= level).unwrap_or(&(0.001, 0.001, 0.0005, "微"));

            // 计算下一个关键价位（根据价格级别调整精度）
            let price_unit = if *interval >= 1.0 {
                *interval / 10.0  // 对于大于1元的价格，使用十分之一作为单位
            } else {
                *interval  // 对于小于1元的价格，使用当前区间作为单位
            };

            let next_key_level = ((current_price / price_unit).floor() + 1.0) * price_unit;
            let distance_to_key = next_key_level - current_price;
            let alert_distance = next_key_level * alert_percent;

            println!("价位分析 - 当前价格: {:.4}, 下一关键位: {:.4}, 距离: {:.4}, 预警距离: {:.4} [{}级别]",
                     current_price, next_key_level, distance_to_key, alert_distance, level_name);

            // 如果接近关键价位且成交量增加，生成卖出信号
            if distance_to_key > 0.0 && distance_to_key < alert_distance && volume_trend.is_increasing {
                // 动态计算建议卖出价格
                let suggested_sell_price = if *interval >= 1.0 {
                    // 大额价格使用百分比回撤
                    next_key_level * (1.0 - pullback_percent)
                } else {
                    // 小额价格使用固定点位回撤
                    next_key_level - (price_unit * pullback_percent)
                };

                // 根据价格级别确定信号类型
                let signal_type = if *interval >= 100.0 { "重要" } else { "普通" };

                println!("价位分析详情:");
                println!("  价格级别: {} (区间: {:.4})", level_name, interval);
                println!("  预警比例: {:.2}%", alert_percent * 100.0);
                println!("  建议回撤: {:.2}%", pullback_percent * 100.0);
                println!("  建议卖价: {:.4}", suggested_sell_price);

                let format_str = if *interval >= 1.0 {
                    format!(
                        "{}价位卖出信号: 当前价格({:.2})接近{}级别关键位({:.2})，建议在{:.2}卖出 [回撤{:.1}%]",
                        signal_type, current_price, level_name, next_key_level, suggested_sell_price,
                        pullback_percent * 100.0
                    )
                } else {
                    format!(
                        "{}价位卖出信号: 当前价格({:.4})接近{}级别关键位({:.4})，建议在{:.4}卖出 [回撤{:.2}%]",
                        signal_type, current_price, level_name, next_key_level, suggested_sell_price,
                        pullback_percent * 100.0
                    )
                };

                return Some(format_str);
            }

            None
        }

        // 新增方法：检查突破条件
    fn check_breakthrough_conditions(&self, data: &[DataItem], ema2_value: f64) -> (bool, bool) {
            if data.len() < 2 {
                return (false, false);
            }
            let current_price = data.last().unwrap().close();
            let prev_price = data[data.len() - 2].close();

            // 向上突破条件：当前价格突破上轨，且前一根K线价格低于EMA2
            let price_above = current_price > ema2_value * (1.0 + self.ema_signal.ema_breakthrough_threshold) && prev_price < ema2_value;

            // 向下突破条件：当前价格突破下轨，且前一根K线价格高于EMA2
            let price_below = current_price < ema2_value * (1.0 - self.ema_signal.ema_breakthrough_threshold) && prev_price > ema2_value;

            (price_above, price_below)
        }
    }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_price_level_sell() {
        let indicator = VegasIndicator::new(12, 26, 50, 576,676);
        let volume_trend = VolumeTrend {
            is_increasing: true,
            is_decreasing: false,
            is_stable: false,
            volume_ratio: 0.0,
        };

        // 测试不同价格区间的情况
        let test_cases = vec![
            // (当前价格, 期望的关键价位, 期望包含的文本)
            (9980.0, 10000.0, "万元级别"),
            (1990.0, 2000.0, "千元级别"),
            (198.0, 200.0, "百元级别"),
            (19.95, 20.0, "十元级别"),
            (1.98, 2.0, "元级别"),
            (0.098, 0.1, "角级别"),
            (0.0098, 0.01, "分级别"),
            (0.00098, 0.001, "厘级别"),
        ];

        for (price, expected_level, expected_text) in test_cases {
            if let Some(signal) = indicator.check_key_price_level_sell(price, &volume_trend) {
                println!("测试价格 {}: {}", price, signal);
                assert!(signal.contains(expected_text), 
                    "价格 {} 应该识别为 {} 级别", price, expected_text);
                assert!(signal.contains(&format!("{:.1}", expected_level)),
                    "价格 {} 的关键价位应该是 {}", price, expected_level);
            } else {
                panic!("价格 {} 应该产生卖出信号", price);
            }
        }
    }
}
