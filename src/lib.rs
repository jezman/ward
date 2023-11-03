use regex::Regex;
use reqwest::{Client, Error};
use std::{collections::HashMap, hash::Hash};

pub struct Camera {
    pub ip: String,
    pub username: String,
    pub password: String,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Car<'a> {
    pub number: &'a str,
    pub begin_date: &'a str,
    pub end_date: &'a str,
}

impl Camera {
    /// return base_usl, example: http://192.168.1.1/cgi-bin
    fn base_url(&self) -> String {
        let url = format!("http://{}/cgi-bin", &self.ip);
        url
    }

    /// Get all numbers from camera in raw camera format four lines for one
    /// car number.
    /// Number<sequence>=carNumber
    /// Begin<sequence>=2020-12-22
    /// End<sequence>=2020-12-22
    /// Notify<sequence>=on | off
    ///
    /// GET request
    /// http://192.168.1.1/cgi-bin/lnpr_cgi?action=list
    ///
    /// return raw camera response list
    pub async fn list_numbers(&self) -> Result<String, Error> {
        let base_url = &self.base_url();
        let request_url = format!("{base_url}/lnpr_cgi?action=list");
        let cars_list_row = &self.get_response(&request_url).await?;

        Ok(cars_list_row.to_string())
    }

    /// Parsing numbers list
    ///
    /// Raw format:
    /// Number507=X111XX777
    /// Begin507=2023-11-02
    /// End507=2023-11-02
    /// Notify=on|off
    ///
    /// return []Car
    pub fn parse_raw_numbers<'a>(&'a self, raw_data: &'a str) -> HashMap<u16, Car> {
        let strings: Vec<&str> = raw_data.split("\n").collect();
        // let numbers_count = (strings.len() / 4) - 1;
        // let mut cars: [Car; 0] = [];
        let mut tmp_cars: HashMap<u16, Car> = HashMap::new();

        let re_sequence = Regex::new(r"[-]?\d[\d,]*[\.]?[\d{1}]*").unwrap();

        for line in strings {
            let number_count: u16;

            let line: Vec<&str> = line.split("=").collect();

            if line[0].contains(&"Number") {
                let submatch_all: Vec<u16> = re_sequence
                    .find_iter(line[0])
                    .filter_map(|digit| digit.as_str().parse::<u16>().ok())
                    .collect();

                number_count = submatch_all[0];
                let car_number = line[1];

                let car = Car {
                    number: car_number,
                    begin_date: "",
                    end_date: "",
                };

                tmp_cars.insert(number_count, car);
            }
            if line[0].contains(&"Begin") {
                let submatch_all: Vec<u16> = re_sequence
                    .find_iter(line[0])
                    .filter_map(|digit| digit.as_str().parse::<u16>().ok())
                    .collect();

                if let Some(car) = tmp_cars.get_mut(&submatch_all[0]) {
                    let begin_date = line[1];
                    car.begin_date = begin_date;
                }
            }
            if line[0].contains(&"End") {
                let submatch_all: Vec<u16> = re_sequence
                    .find_iter(line[0])
                    .filter_map(|digit| digit.as_str().parse::<u16>().ok())
                    .collect();

                if let Some(car) = tmp_cars.get_mut(&submatch_all[0]) {
                    let end_date = line[1];
                    car.end_date = end_date;
                }
            }
        }

        tmp_cars
    }

    /// Send request to camera and get response
    /// For request need set base_auth
    async fn get_response(&self, url: &str) -> Result<String, Error> {
        let client = Client::new();
        let response = client
            .get(url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                println!("Success!");
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                println!("Unathorized");
            }
            _ => {
                panic!("Uh oh! Something unexpected happened.");
            }
        };
        Ok(response.text().await?)
    }
}
