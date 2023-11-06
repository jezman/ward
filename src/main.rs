use ward::{Camera, Car};

use std::env;

fn help() {
    println!(
        "Использование:
ward list
    Список всех номеров в камере.
ward clear 2023-11-22
    Удалить все номера на указанную дату.
ward remove X111XX777
    Удалить номер из камеры.
ward add|edit X111XX777 2023-11-22 22-11-23
    Добавить, редактировать номер."
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let camera = Camera::new();

    match args.len() {
        1 => {
            help();
        }
        2 => {
            let command = &args[1];
            match &command[..] {
                "list" => {
                    println!("|№     | Номер авто      | Начало     | Конец      |");
                    match camera.list_numbers() {
                        Ok(cars) => {
                            for (i, car) in cars.iter().enumerate() {
                                println!(
                                    "|{:<5} | {:<15} | {:^10} | {:^10} |",
                                    i + 1,
                                    car.number,
                                    car.begin_date,
                                    car.end_date
                                )
                            }
                        }
                        Err(e) => println!("{}", e),
                    }
                }
                "help" => help(),
                _ => {
                    eprintln!("ошибка: неверная команда");
                    help();
                }
            };
        }
        3 => {
            let command = &args[1];
            let arg = &args[2].to_uppercase();

            let car = Car {
                number: arg.to_string(),
                begin_date: "".to_string(),
                end_date: "".to_string(),
            };

            match &command[..] {
                "clear" => {
                    match camera.remove_cars(arg.to_string()) {
                        Ok(r) => println!("{:?}", r),
                        Err(e) => println!("{}", e),
                    }
                },
                "remove" => {
                    match camera.remove(&car)  {
                        Ok(r) => println!("{:?}", r.trim()),
                        Err(e) => println!("{}", e),
                    }
                },    
                _ => {
                    eprintln!("ошибка: неверная команда");
                    help();
                }
            }
        }
        5 => {
            let command = &args[1];
            let number = &args[2].to_uppercase();
            let begin_date = &args[3];
            let end_date = &args[4];

            let car = Car {
                number: number.to_string(),
                begin_date: begin_date.to_string(),
                end_date: end_date.to_string(),
            };
            // parse the command
            match &command[..] {
                "add" => {
                    match camera.add(&car)  {
                        Ok(r) => println!("{:?}", r.trim()),
                        Err(e) => println!("{}", e),
                    }
                },    
                "edit" => {
                    match camera.edit(&car)  {
                        Ok(r) => println!("{:?}", r.trim()),
                        Err(e) => println!("{}", e),
                    }
                },    
                "help" => help(),
                _ => {
                    eprintln!("ошибка: неверная команда");
                    help();
                }
            };
        }
        // all the other cases
        _ => {
            // show a help message
            help();
        }
    }
}
