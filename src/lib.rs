use chrono::offset::Local;
use regex::Regex;
use reqwest::{blocking::Client, Error};
use std::{process::exit, thread, time};

pub struct Camera {
    pub ip: String,
    pub username: String,
    pub password: String,
}

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
    /// Get all cars numbers from camera.
    pub fn list_numbers(&self) -> Result<Vec<Car>, Error> {
        let path = "/lnpr_cgi?action=list";
        let raw_cars_list = &self.get_response(&path)?;

        let cars_list = self.parse_raw_cars_list(&raw_cars_list)?;

        Ok(cars_list)
    }

    /// Add number to camera
    /// allowed symbols for number: ABCEHKMOPTXY0-9
    /// dates format 2023-11-22
    pub fn add(&self, car: &mut Car) -> Result<String, Error> {
        if car.begin_date.is_empty() || car.end_date.is_empty() {
            car.begin_date = Local::now().format("%Y-%m-%d").to_string();
            car.end_date = Local::now().format("%Y-%m-%d").to_string();
        }

        let path = format!(
            "/lnpr_cgi?action=add&Number={}&Begin={}&End={}",
            car.number, car.begin_date, car.end_date,
        );
        let response = &self.get_response(&path)?;
        Ok(response.trim().to_string())
    }

    /// Edit number in camera
    /// allowed symbols for number: ABCEHKMOPTXY0-9,
    /// dates format 2023-11-22
    pub fn edit(&self, car: &Car) -> Result<String, Error> {
        let path = format!(
            "/lnpr_cgi?action=edit&Number={}&Begin={}&End={}",
            car.number, car.begin_date, car.end_date,
        );
        let response = &self.get_response(&path)?;
        Ok(response.trim().to_string())
    }

    /// Remove number from camera
    /// allowed symbols for number: ABCEHKMOPTXY0-9,
    /// dates format 2023-11-22
    pub fn remove(&self, car: &Car) -> Result<String, Error> {
        let path = format!("/lnpr_cgi?action=remove&Number={}", car.number);
        let response = &self.get_response(&path)?;
        Ok(response.trim().to_string())
    }

    /// Remove all numbers by end_date,
    /// dates format 2023-11-22
    pub fn remove_cars(&self, end_date: String) -> Result<String, Error> {
        let cars = self.list_numbers()?;
        println!("CRON JOB: удаление номеров автомобилей за текущий день");
        for car in cars {
            if car.end_date == end_date {
                let _ = &self.remove(&car)?;
                println!("Номер {} удален", car.number);
                thread::sleep(time::Duration::from_millis(500));
            }
        }
        Ok("Удаление номеров закончено".to_string())
    }

    /// Parsing raw cars list
    ///
    /// Number111=carNumber
    /// Begin111=2020-12-22
    /// End111=2020-12-22
    /// Notify111=on | off
    /// 111 - sequence number in camera
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

    fn get_response(&self, path: &str) -> Result<String, Error> {
        let base_path = format!("http://{}/cgi-bin", &self.ip);
        let url = format!("{}{}", base_path, path);

        let client = Client::new();
        let response = client
            .get(url)
            .basic_auth(&self.username, Some(&self.password))
            .timeout(std::time::Duration::from_millis(3000))
            .send()?;

        match response.status() {
            reqwest::StatusCode::OK => {
                return Ok(response.text()?);
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                eprintln!("вы не авторизованы");
            }
            _ => {
                eprintln!("произошла ошибка: {}", response.text()?);
                exit(1);
            }
        };
        Ok(response.text()?)
    }
}

/// Finding digit in string
fn find_digits(string: &str) -> Vec<usize> {
    let re_sequence = Regex::new(r"[-]?\d[\d,]*[\.]?[\d{1}]*").unwrap();
    let submatch_all = re_sequence
        .find_iter(string)
        .filter_map(|digit| digit.as_str().parse::<usize>().ok())
        .collect();

    submatch_all
}

fn get_env_var(var_name: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| panic!("переменная {} не найдена", var_name))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env::{remove_var, set_var};
    use {Camera, Car};

    #[test]
    fn test_find_digit() {
        let raw_string = "Number111=carNumber";
        let digit = find_digits(raw_string);
        assert_eq!(digit[0], 111);
    }
    #[test]
    fn test_get_env_var() {
        set_var("WARD_TEST", "VALUE");
        assert_eq!(get_env_var("WARD_TEST"), "VALUE");
        remove_var("WARD_TEST");
    }

    #[test]
    fn add_car_success() {
        let camera = Camera::new();
        let mut car = Car {
            number: "XXXX1112".to_string(),
            begin_date: "2023-12-12".to_string(),
            end_date: "2023-12-12".to_string(),
        };
        if let Ok(res) = camera.add(&mut car) {
            assert_eq!(res, "OK");
        };
    }

    #[test]
    fn edit_car_success() {
        let camera = Camera::new();
        let mut car = Car {
            number: "XXXX1112".to_string(),
            begin_date: "2023-12-22".to_string(),
            end_date: "2023-12-22".to_string(),
        };
        if let Ok(res) = camera.edit(&mut car) {
            assert_eq!(res, "OK");
        };
    }

    #[test]
    fn remove_car_success() {
        let camera = Camera::new();
        let mut car = Car {
            number: "XXXX1112".to_string(),
            begin_date: "".to_string(),
            end_date: "".to_string(),
        };
        if let Ok(res) = camera.remove(&mut car) {
            assert_eq!(res, "OK");
        };
    }
}
