use chrono;
use regex::Regex;
use reqwest::{blocking::Client, Error};
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
    pub fn new() -> Self {
        let ip = get_env_var("CAMERA_IP");
        let username = get_env_var("CAMERA_USERNAME");
        let password = get_env_var("CAMERA_PASSWORD");

        let camera = Camera {
            ip,
            password,
            username,
        };

        camera
    }
    /// Get all numbers from camera cars numbers.
    ///
    /// Number111=carNumber
    /// Begin111=2020-12-22
    /// End111=2020-12-22
    /// Notify111=on | off
    /// 111 - sequence number in camera
    pub fn list_numbers(&self) -> Result<Vec<Car>, Error> {
        let path = "/lnpr_cgi?action=list";
        let raw_cars_list = &self.get_response(&path)?;

        let cars_list = self.parse_raw_cars_list(&raw_cars_list)?;
        Ok(cars_list)
    }

    /// Add number to camera
    /// allowed symbols for number: ABCEHKMOPTXY0-9
    /// dates format 2023-11-22
    pub fn add(&self, car: &Car) -> Result<String, Error> {
        let path = format!(
            "/lnpr_cgi?action=add&Number={}&Begin={}&End={}",
            car.number, car.begin_date, car.end_date,
        );
        let response = &self.get_response(&path)?;
        Ok(response.to_string())
    }

    /// Edit number in camera
    /// allowed symbols: ABCEHKMOPTXY0-9
    pub fn edit(&self, car: &Car) -> Result<String, Error> {
        let path = format!(
            "/lnpr_cgi?action=edit&Number={}&Begin={}&End={}",
            car.number, car.begin_date, car.end_date,
        );
        let response = &self.get_response(&path)?;
        Ok(response.to_string())
    }

    /// Remove number from camera
    /// allowed symbols: ABCEHKMOPTXY0-9
    pub fn remove(&self, car: &Car) -> Result<String, Error> {
        let path = format!("/lnpr_cgi?action=remove&Number={}", car.number);
        let response = &self.get_response(&path)?;
        Ok(response.to_string())
    }

    /// Remove all numbers by end_date
    pub fn remove_cars(&self, mut end_date: String) -> Result<(), Error> {
        if end_date.is_empty() {
            end_date = chrono::offset::Local::now().format("%Y-%m-%d").to_string();
        }

        let cars = self.list_numbers()?;
        println!("CRON JOB: удаление номеров автомобилей за текущий день");
        for car in cars {
            if car.end_date == end_date {
                let _ = &self.remove(&car)?;
                println!("Номер {} удален", car.number);
                thread::sleep(time::Duration::from_millis(500));
            }
        }
        Ok(())
    }

    fn parse_raw_cars_list(&self, raw_cars_list: &str) -> Result<Vec<Car>, Error> {
        let strings: Vec<&str> = raw_cars_list.split("\n").collect();
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

    pub fn get_response(&self, path: &str) -> Result<String, Error> {
        let base_path = format!("http://{}/cgi-bin", &self.ip);
        let url = format!("{}{}", base_path, path);

        let client = Client::new();
        let response = client
            .get(url)
            .basic_auth(&self.username, Some(&self.password))
            .send()?;

        match response.status() {
            reqwest::StatusCode::OK => {
                return Ok(response.text()?);
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                println!("Вы не авторизованы");
            }
            _ => {
                panic!("Произошла ошибка");
            }
        };
        Ok(response.text()?)
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

fn get_env_var(var_name: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| panic!("Переменная {} не найдена", var_name))
}