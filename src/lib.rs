use chrono;
use regex::Regex;
use reqwest::{Client, Error};
use std::{thread, time};

pub struct Camera {
    pub ip: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct Car {
    pub number: String,
    pub begin_date: String,
    pub end_date: String,
}

impl Camera {
    /// return base_usl, example: http://192.168.1.1/cgi-bin
    async fn base_url(&self) -> String {
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
    /// return camera vector parsed response list
    pub async fn list_numbers(&self) -> Result<Vec<Car>, Error> {
        let base_url = &self.base_url().await;
        let request_url = format!("{base_url}/lnpr_cgi?action=list");
        let cars_list_row = &self.get_response(&request_url).await?;

        let strings: Vec<&str> = cars_list_row.split("\n").collect();
        let mut cars: Vec<Car> = vec![];

        for line in strings {
            let number_count: usize;
            let line: Vec<&str> = line.split("=").collect();

            if line[0].contains(&"Number") {
                let submatch_all = find_digits(line[0]);
                number_count = submatch_all[0];
                let car_number = line[1];

                let car = Car {
                    number: car_number.to_string(),
                    begin_date: "".to_string(),
                    end_date: "".to_string(),
                };

                cars.insert(number_count, car);
            }

            if line[0].contains(&"Begin") {
                let submatch_all = find_digits(line[0]);
                let begin_date = line[1];
                let car = &mut cars[submatch_all[0]];
                car.begin_date = begin_date.to_string();
            }
            if line[0].contains(&"End") {
                let submatch_all = find_digits(line[0]);
                let end_date = line[1];
                let car = &mut cars[submatch_all[0]];
                car.end_date = end_date.to_string();
            }
        }

        Ok(cars)
    }

    /// Add number to camera
    /// allowed symbols for number: ABCEHKMOPTXY0-9
    /// dates format 2023-11-22
    pub async fn add(&self, car: &Car) -> Result<String, Error> {
        let base_url = &self.base_url().await;
        let url = format!(
            "{}/lnpr_cgi?action=add&Number={}&Begin={}&End={}",
            base_url, car.number, car.begin_date, car.end_date,
        );
        let response = &self.get_response(&url).await?;
        Ok(response.to_string())
    }

    /// Edit number in camera
    /// allowed symbols: ABCEHKMOPTXY0-9
    pub async fn edit(&self, car: &Car) -> Result<String, Error> {
        let base_url = &self.base_url().await;
        let url = format!(
            "{}/lnpr_cgi?action=edit&Number={}&Begin={}&End={}",
            base_url, car.number, car.begin_date, car.end_date,
        );
        let response = &self.get_response(&url).await?;
        Ok(response.to_string())
    }

    /// Remove number from camera
    /// allowed symbols: ABCEHKMOPTXY0-9
    pub async fn remove(&self, car: &Car) -> Result<String, Error> {
        let base_url = &self.base_url().await;
        let url = format!("{}/lnpr_cgi?action=remove&Number={}", base_url, car.number);
        let response = &self.get_response(&url).await?;
        Ok(response.to_string())
    }

    /// Remove all numbers by end_date
    pub async fn remove_cars(&self, mut end_date: String) -> Result<(), Error> {
        if end_date.is_empty() {
            end_date = chrono::offset::Local::now().format("%Y-%m-%d").to_string();
        }

        let cars = &self.list_numbers().await?;
                println!("CRON JOB: удаление номеров автомобилей за текущий день");
                for car in cars {
                    if car.end_date == end_date {
                        let _ = &self.remove(car).await?;
                        println!("Номер {} удален", car.number);
                        thread::sleep(time::Duration::from_millis(500));
                    }
                }
        Ok(())
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
                return Ok(response.text().await?);
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                println!("Вы не авторизованы");
            }
            _ => {
                panic!("Произошла ошибка");
            }
        };
        Ok(response.text().await?)
    }
}

fn find_digits(string: &str) -> Vec<usize> {
    let re_sequence = Regex::new(r"[-]?\d[\d,]*[\.]?[\d{1}]*").unwrap();
    let submatch_all: Vec<usize> = re_sequence
        .find_iter(string)
        .filter_map(|digit| digit.as_str().parse::<usize>().ok())
        .collect();

    submatch_all
}
