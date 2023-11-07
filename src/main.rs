use std::{env, process::exit};
use ward::{Camera, Car};

fn help() {
    println!("
    #     #                      
    #  #  #   ##   #####  #####  
    #  #  #  #  #  #    # #    # 
    #  #  # #    # #    # #    # 
    #  #  # ###### #####  #    # 
    #  #  # #    # #   #  #    # 
     ## ##  #    # #    # #####  
                                     
Управление номерами автомобилей в камере Beward B2530RZQ-LP 

Использование:

ward list
    Список всех номеров в камере.
ward clear 2023-11-22
    Удалить все номера на указанную дату.
ward add X111XX777
    Добавить номер на сегодня.
ward remove X111XX777
    Удалить номер из камеры.
ward add|edit X111XX777 2023-11-22 22-11-23
    Добавить или редактировать номер с указанными датами."
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
                "list" => match camera.list_numbers() {
                    Ok(cars) => {
                        println!("|№     | Номер авто      | Начало     | Конец      |");

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
                    Err(err) => {
                        eprintln!("не удалось получить список номеров\n{}", err);
                        exit(1);
                    }
                },
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

            let mut car = Car {
                number: arg.to_string(),
                begin_date: "".to_string(),
                end_date: "".to_string(),
            };

            match &command[..] {
                "add" => {
                    if let Err(err) = camera.add(&mut car) {
                        eprintln!("не удалось добавить номер\n{}", err);
                        exit(1);
                    }
                }
                "clear" => {
                    if let Err(err) = camera.remove_cars(arg.to_string()) {
                        eprintln!("не удалось удалить номер\n{}", err);
                        exit(1);
                    }
                }
                "remove" => {
                    if let Err(err) = camera.remove(&car) {
                        eprintln!("не удадалось удалить номера\n{}", err);
                        exit(1);
                    }
                }
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

            let mut car = Car {
                number: number.to_string(),
                begin_date: begin_date.to_string(),
                end_date: end_date.to_string(),
            };

            match &command[..] {
                "add" => {
                    if let Err(err) = camera.add(&mut car) {
                        eprintln!("не удалось добавить номер\n{}", err);
                        exit(1);
                    }
                }
                "edit" => {
                    if let Err(err) = camera.edit(&car) {
                        eprintln!("не удалось отредактировать номер\n{}", err);
                        exit(1);
                    }
                }
                "help" => help(),
                _ => {
                    eprintln!("ошибка: неверная команда");
                    help();
                }
            };
        }
        _ => {
            help();
        }
    }
}
