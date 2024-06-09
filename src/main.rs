use csv::Reader;
use std::error::Error;

fn read_csv(file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut rdr = Reader::from_path(file_path)?;

    for result in rdr.records() {
        let record = result?;

        let open:i32 = record[1].parse::<i32>()?;
        println!("{:?}", open);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    read_csv("test.csv")?;
    Ok(())
}
