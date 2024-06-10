use csv::Reader;
use plotly::{Plot, Scatter};
use statistics::mean;
use statistics::variance;
use std::error::Error;

fn read_csv(file_path: &str) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), Box<dyn Error>> {
    let mut rdr = Reader::from_path(file_path)?;

    let mut open_vec: Vec<f64> = vec![];
    let mut high_vec: Vec<f64> = vec![];
    let mut low_vec: Vec<f64> = vec![];
    let mut close_vec: Vec<f64> = vec![];

    for result in rdr.records() {
        let record = result?;
        let open: f64 = record[1].parse::<f64>()?;
        let high: f64 = record[2].parse::<f64>()?;
        let low: f64 = record[3].parse::<f64>()?;
        let close: f64 = record[4].parse::<f64>()?;
        open_vec.push(open);
        high_vec.push(high);
        low_vec.push(low);
        close_vec.push(close);
    }
    Ok((open_vec, high_vec, low_vec, close_vec))
}

fn strategy(
    pi_open: Vec<f64>,
    pi_high: Vec<f64>,
    pi_low: Vec<f64>,
    _pi_close: Vec<f64>,
    btc_open: Vec<f64>,
    _btc_high: Vec<f64>,
    _btc_low: Vec<f64>,
    btc_close: Vec<f64>,
    window_size: usize,
    bb_multiplier: f64,
) -> Vec<f64> {
    let pi_higher: Vec<f64> = calculate_rolling_mean(&pi_high, window_size)
        .iter()
        .zip(calculate_rolling_std(&pi_high, window_size).iter())
        .map(|(&mean, &std)| mean + bb_multiplier * std)
        .collect();

    let pi_lower: Vec<f64> = calculate_rolling_mean(&pi_low, window_size)
        .iter()
        .zip(calculate_rolling_std(&pi_low, window_size).iter())
        .map(|(&mean, &std)| mean - bb_multiplier * std)
        .collect();

    let mut btc_entry_signal: Vec<i64> = vec![0; pi_open.len()];
    let mut btc_signal_with_exits: Vec<i64> = vec![0; pi_open.len()];
    let mut btc_position: Vec<i64> = vec![0; pi_open.len()];
    let mut btc_pnl: Vec<f64> = vec![0.0; pi_open.len()];
    let btc_mid_line: Vec<f64> = pi_higher
        .iter()
        .zip(pi_lower.iter())
        .map(|(&a, &b)| (a + b) / 2.0)
        .collect();

    println!("{}", (pi_higher.len()));
    for i in 2..(pi_open.len() - 1) {
        if (pi_high[i - 2] < pi_higher[i - 2])
            && (pi_high[i - 1] > pi_higher[i - 1])
            && (btc_position[i - 1] == 0)
        {
            btc_entry_signal[i] = -1;
            btc_position[i] = btc_position[i - 1] - 1;
        } else if (pi_low[i - 2] > pi_lower[i - 2])
            && (pi_low[i - 1] < pi_lower[i - 1])
            && (btc_position[i - 1] == 0)
        {
            btc_entry_signal[i] = 1;
            btc_position[i] = btc_position[i - 1] + 1;
        } else {
            btc_entry_signal[i] = 0;
            btc_position[i] = btc_position[i - 1];
        };
        // Exit condition
        if (btc_position[i - 1] == 1)
            && (pi_low[i - 2] < btc_mid_line[i - 2])
            && (pi_low[i - 1] > btc_mid_line[i - 1])
        {
            btc_position[i] = 0;
        } else if (btc_position[i - 1] == -1)
            && (pi_high[i - 2] > btc_mid_line[i - 2])
            && (pi_high[i - 1] < btc_mid_line[i - 1])
        {
            btc_position[i] = 0;
        };
        // Signal with exit
        if (btc_entry_signal[i] == -1) {
            btc_signal_with_exits[i] = btc_entry_signal[i];
        } else if (btc_entry_signal[i] == 1) {
            btc_signal_with_exits[i] = btc_entry_signal[i];
        } else if (btc_entry_signal[i] == 0)
            && (btc_position[i - 1] == -1)
            && (btc_position[i] == 0)
        {
            btc_signal_with_exits[i] = 1;
        } else if (btc_entry_signal[i] == 0) && (btc_position[i - 1] == 1) && (btc_position[i] == 0)
        {
            btc_signal_with_exits[i] = -1;
        } else {
            btc_signal_with_exits[i] = 0;
        };
        // Calculate PnL
        if btc_position[i] == -1 {
            let entry_price: f64 = btc_open[i];
            let exit_price: f64 = btc_close[i];
            btc_pnl[i] = -((exit_price / entry_price) - 1.0);
        } else if btc_position[i] == 1 {
            let entry_price: f64 = btc_open[i];
            let exit_price: f64 = btc_close[i];
            btc_pnl[i] = (exit_price / entry_price) - 1.0;
        };
        // Transaction cost
        if (btc_signal_with_exits[i] == 1) | (btc_signal_with_exits[i] == -1) {
            btc_pnl[i] = btc_pnl[i] - 0.06 / 100.0;
        }
    }
    btc_pnl
}

fn main() -> Result<(), Box<dyn Error>> {
    let result = read_csv("PI_1H.csv")?;
    let pi_open: Vec<f64> = result.0;
    let pi_high: Vec<f64> = result.1;
    let pi_low: Vec<f64> = result.2;
    let pi_close: Vec<f64> = result.3;

    let btc_data = read_csv("BTC_price_1H.csv")?;
    let btc_open: Vec<f64> = btc_data.0;
    let btc_high: Vec<f64> = btc_data.1;
    let btc_low: Vec<f64> = btc_data.2;
    let btc_close: Vec<f64> = btc_data.3;

    let mut count: Vec<usize> = Vec::new();
    for i in 0..(pi_high.len()) {
        count.push(i);
    }

    let window_size: usize = 100;
    let bb_multiplier: f64 = 2.0;

    let btc_pnl: Vec<f64> = strategy(
        pi_open,
        pi_high,
        pi_low,
        pi_close,
        btc_open,
        btc_high,
        btc_low,
        btc_close,
        window_size,
        bb_multiplier,
    );
    // println!("{:?}", btc_pnl);
    let btc_pnl_cum_sum: Vec<f64> = calculate_cumulative_sum(&btc_pnl);
    let mut plot = Plot::new();
    let trace = Scatter::new(count, btc_pnl_cum_sum);
    plot.add_trace(trace);
    plot.show();

    Ok(())
}

fn calculate_rolling_mean(data: &[f64], window_size: usize) -> Vec<f64> {
    let mut rolling_mean: Vec<f64> = data
        .windows(window_size)
        .map(|window| mean(window))
        .collect();
    for i in 0..(window_size - 1) {
        rolling_mean.insert(i, std::f64::NAN);
    }
    rolling_mean
}

fn calculate_rolling_std(data: &[f64], window_size: usize) -> Vec<f64> {
    let mut rolling_std: Vec<f64> = data
        .windows(window_size)
        .map(|window| variance(window).sqrt())
        .collect();
    for i in 0..(window_size - 1) {
        rolling_std.insert(i, std::f64::NAN);
    }
    rolling_std
}

fn calculate_cumulative_sum(numbers: &[f64]) -> Vec<f64> {
    numbers
        .iter()
        .scan(0.0, |acc, &x| {
            *acc += x;
            Some(*acc)
        })
        .collect()
}

fn strategy_test(
    _pi_open: Vec<f64>,
    pi_high: Vec<f64>,
    pi_low: Vec<f64>,
    _pi_close: Vec<f64>,
    _btc_open: Vec<f64>,
    _btc_high: Vec<f64>,
    _btc_low: Vec<f64>,
    _btc_close: Vec<f64>,
    window_size: usize,
    bb_multiplier: f64,
) -> Vec<f64> {
    let pi_higher: Vec<f64> = calculate_rolling_mean(&pi_high, window_size)
        .iter()
        .zip(calculate_rolling_std(&pi_high, window_size).iter())
        .map(|(&mean, &std)| mean + bb_multiplier * std)
        .collect();

    let pi_lower: Vec<f64> = calculate_rolling_mean(&pi_low, window_size)
        .iter()
        .zip(calculate_rolling_std(&pi_low, window_size).iter())
        .map(|(&mean, &std)| mean + bb_multiplier * std)
        .collect();

    let btc_mid_line: Vec<f64> = pi_higher
        .iter()
        .zip(pi_lower.iter())
        .map(|(&a, &b)| (a + b) / 2.0)
        .collect();

    let roll_mean: Vec<f64> = calculate_rolling_mean(&pi_high, 100);

    println!("{}", pi_higher.len());
    println!("{}", roll_mean.len());
    btc_mid_line
}
