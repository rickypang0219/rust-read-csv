use csv::Reader;
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

fn main() -> Result<(), Box<dyn Error>> {
    let result = read_csv("PI_1H.csv")?;
    let open_prices: Vec<f64> = result.0;
    let high_prices: Vec<f64> = result.1;
    let low_prices: Vec<f64> = result.2;
    let close_prices: Vec<f64> = result.3;

    for price in open_prices {
        println!("{}", price);
    }
    Ok(())
}
