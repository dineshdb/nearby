use bluer::{
    AdapterEvent, Address, DeviceEvent, DeviceProperty, DiscoveryFilter, DiscoveryTransport,
};
use config::{get_config, Connection};
use futures::{pin_mut, stream::SelectAll, StreamExt};
use std::collections::HashSet;
mod config;
mod delayed;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = get_config()?;
    let ble_addresses: HashSet<_> = config
        .connections()
        .iter()
        .filter_map(|c| c.get_ble())
        .map(|c| c.mac.parse::<Address>().unwrap())
        .collect();

    env_logger::init();
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let filter = DiscoveryFilter {
        transport: DiscoveryTransport::Le,
        duplicate_data: true,
        ..Default::default()
    };
    adapter.set_discovery_filter(filter).await?;
    let device_events = adapter.discover_devices().await?;
    pin_mut!(device_events);

    let mut all_change_events = SelectAll::new();

    loop {
        tokio::select! {
            Some(device_event) = device_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        if !ble_addresses.is_empty() && !ble_addresses.contains(&addr) {
                            continue;
                        }
                        let device = adapter.device(addr)?;
                        let rssi = device.rssi().await?.unwrap_or_default();
                        let distance = distance_rssi(rssi);
                        let connection  = config.get_connection_by_mac(&addr.to_string());
                        if let Some(ble_connection) = connection.and_then(Connection::get_ble) {
                            ble_connection.run_proximity_actions(distance);
                            println!(
                                "{:?} {:?} {:.2}m",
                                addr,
                                ble_connection.name,
                                distance,
                            );
                        }

                        // with changes
                        let device = adapter.device(addr)?;
                        let change_events = device.events().await?.map(move |evt| (addr, evt));
                        all_change_events.push(change_events);
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
                        let distance = 1000.0;
                        let connection  = config.get_connection_by_mac(&addr.to_string());
                        if let Some(ble_connection) = connection.and_then(Connection::get_ble){
                            println!("{addr} {} {distance:.2}m Removed", ble_connection.name);
                            // todo: add proper logging
                            // todo: run delayed device lock just in case the device comes back online again
                            ble_connection.run_proximity_actions(distance);

                        } else {
                            println!("{addr} Removed");
                        }
                    }
                    _ => (),
                }
            }
            Some((addr, DeviceEvent::PropertyChanged(property))) = all_change_events.next() => {
                match property {
                    DeviceProperty::Rssi(rssi) => {
                        let connection  = config.get_connection_by_mac(&addr.to_string()).unwrap();
                        let ble_connection = connection.get_ble().expect("ble connection expected");
                        let distance = distance_rssi(rssi);
                        ble_connection.run_proximity_actions(distance);

                        println!(
                            "{:?} {:?} {:.2}m",
                            addr,
                            ble_connection.name,
                            distance,
                        );
                    },
                    _ => {
                        // println!("    {property:?}");
                    }
                }
            }
            else => break
        }
    }

    Ok(())
}

pub fn distance_rssi(rssi: i16) -> f32 {
    // 10 ^ ((-69 – (-60))/(10 * 2))
    let exponent = (-69 - rssi) as f32 / (10_i16.pow(2)) as f32;
    10_f32.powf(exponent)
}
