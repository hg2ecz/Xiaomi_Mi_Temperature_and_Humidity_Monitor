use blurz::{BluetoothDevice, BluetoothEvent, BluetoothGATTCharacteristic, BluetoothSession};

fn help() {
    eprintln!("Usage:");
    eprintln!("-q quiet for automated data processing");
    eprintln!("-q10 quiet and temperature multiplied by 10 for MRTG");
    eprintln!("A4:C1:38:12:34:56 ... bluetooth ids");
    eprintln!();
    std::process::exit(0);
}

fn main() {
    if std::env::args().len() < 2 {
        help();
    }

    let mut devices = false;
    let bt_session = &BluetoothSession::create_session(None).unwrap();

    let mut quiet = false;
    let mut temp_mul10 = false;

    let mut dev_len = 0;
    for dev in std::env::args().skip(1) {
        if dev == "-q" {
            quiet = true;
            continue;
        }

        if dev == "-q10" {
            quiet = true;
            temp_mul10 = true;
            continue;
        }

        if !quiet {
            println!("dev: {}", dev);
        }
        for _ in 0..5 {
            let device = BluetoothDevice::new(bt_session, format!("/org/bluez/hci0/dev_{}", dev.replace(&":", "_")));
            match device.connect(10000) {
                Err(_e) => (), // println!("Failed to connect {:?}: {:?}", device.get_id(), _e),
                Ok(_) => {
                    let temp_humidity = BluetoothGATTCharacteristic::new(bt_session, device.get_id() + "/service0021/char0035");
                    if temp_humidity.start_notify().is_ok() {
                        devices = true;
                        dev_len += 1;
                        break;
                    }
                }
            }
        }
    }

    if devices {
        let mut devid_list = vec![];
        for event in BluetoothSession::create_session(None).unwrap().incoming(10000).map(BluetoothEvent::from) {
            if event.is_none() {
                continue;
            }

            let (device_id, value) = match event.clone().unwrap() {
                BluetoothEvent::Value { object_path, value } => (
                    object_path
                        .replace(&"/org/bluez/hci0/dev_", "")
                        .replace(&"/service0021/char0035", "")
                        .replace(&"_", ":"),
                    value,
                ),
                _ => continue,
            };

            if !devid_list.contains(&device_id) {
                let mut temperature_array = [0; 2];
                temperature_array.clone_from_slice(&value[..2]);
                let temperature = u16::from_le_bytes(temperature_array) as f32 * 0.01;
                let humidity = value[2];

                if quiet {
                    if temp_mul10 {
                        println!("{} {} {:?}", device_id, (10. * temperature) as i32, humidity);
                    } else {
                        println!("{} {:.2} {:?}", device_id, temperature, humidity);
                    }
                } else {
                    println!("Device: {}, Temperature: {:.2}°C Humidity: {:?}%", device_id, temperature, humidity);
                }
                devid_list.push(device_id);
            }

            if devid_list.len() >= dev_len {
                std::process::exit(0);
            }
        }
    }
    std::process::exit(1);
}
