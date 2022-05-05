use anyhow::{Context, Ok, Result};
use clap::Parser;
use csv::StringRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Cli {
    /// The PO csv file to be used
    #[clap(short, long)]
    input: PathBuf,
    /// The destination directory where the processed POs will be saved
    #[clap(short, long)]
    output: PathBuf,
    /// The text file that contains all of the style numbers to be processed
    #[clap(short, long)]
    list: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
#[serde(rename_all = "PascalCase")]
struct Shits {
    style: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
#[serde(rename_all = "PascalCase")]
struct Order {
    po: String,
    style_code: String,
    color_code: String,
    msrp_size: String,
    style_desc: String,
    color_desc: String,
    upc: String,
    store_num: String,
    qty: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
#[serde(rename_all = "PascalCase")]
struct Report {
    po: String,
    store: String,
    style_code: String,
    size: String,
    qty: String,
}

fn read_file(file_path: PathBuf) -> Result<Vec<StringRecord>> {
    let file = File::open(file_path).context("Failed to open file")?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records: Vec<StringRecord> = vec![];

    for result in rdr.records() {
        records.push(result?);
    }

    Ok(records)
}

fn list(path: PathBuf) -> Vec<String> {
    let file = std::fs::read_to_string(path)
        .expect("Could not read the file containing the stores to search for, check file")
        .lines()
        .collect::<String>();

    let file = file
        .split(",")
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();
    file
}

fn find_styles(records: Vec<StringRecord>, list: Vec<String>) -> Result<Vec<StringRecord>> {
    let mut filtered_records = vec![];

    for style in list {
        for item in records.clone().into_iter() {
            // Combines article and color
            let style_and_color =
                item.get(1).unwrap().to_owned() + &item.get(2).unwrap().to_owned();
            if style_and_color.contains(&style) {
                filtered_records.push(item)
            }
        }
    }

    Ok(filtered_records)
}
fn write_file(records: Vec<StringRecord>, destination_path: PathBuf) -> Result<()> {
    // By using a HashSet, we remove all duplicated records from the vector.
    // We aquire a set of unique POs that we can use as file names below.
    let store_list = records
        .iter()
        .map(|num| num.get(0).unwrap().to_owned())
        .collect::<HashSet<String>>();

    let file_path = destination_path;

    let file_name = dbg!(file_path.join(format!("{}.csv", "Report")));
    let mut wtr = csv::Writer::from_writer(File::create(file_name)?);

    for each in records.iter() {
        wtr.serialize(Report {
            po: each.get(0).unwrap().to_owned(),
            store: each.get(7).unwrap().to_owned(),
            style_code: each.get(1).unwrap().to_owned() + &each.get(2).unwrap().to_owned(),
            size: each.get(3).unwrap().to_owned(),
            qty: each.get(8).unwrap().to_owned(),
        })?;
    }
    wtr.flush()?;

    Ok(())
}
fn main() -> Result<()> {
    let args = Cli::parse();

    let store_list: Vec<String> = list(args.list);
    let results = read_file(args.input)?;
    let results = find_styles(results, store_list)?;
    write_file(results, args.output)?;
    Ok(())
}
