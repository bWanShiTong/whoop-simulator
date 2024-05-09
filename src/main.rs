use futures::{channel::mpsc::channel, prelude::*};
use std::{
    collections::HashSet,
    sync::{atomic, Arc, Mutex},
    thread,
    time::Duration,
};
use tokio::time::sleep;
use uuid::{uuid, Uuid};

use bluster::{
    gatt::{
        characteristic::{self, Characteristic, Properties, Read, Secure, Write},
        descriptor::{self, Descriptor},
        event::{Event, NotifySubscribe, Response},
        service::Service,
    },
    uuid_from_sdp, Error, Peripheral, SdpShortUuid,
};

const ADVERTISING_NAME: &str = "WHOOP RE";
const SPECIAL_SAUCE: Uuid = uuid!("61080001-8d6d-82b8-614a-1c8cb0f8dcc6");
const HEART_RATE: Uuid = uuid_from_sdp(0x180D);
const BOND: Uuid = uuid_from_sdp(0x181E);

fn cmd_to_strap_char() -> Characteristic {
    let (write, mut read) = channel(1);

    tokio::spawn(async move {
        while let Some(data) = read.next().await {
            dbg!(data);
        }
    });

    Characteristic::new(
        uuid!("61080002-8d6d-82b8-614a-1c8cb0f8dcc6"),
        Properties::new(None, Some(Write::WithoutResponse(write)), None, None),
        None,
        HashSet::default(),
    )
}

fn battery_service() -> Service {
    let (write, mut read) = channel(1);

    tokio::spawn(async move {
        while let Some(data) = read.next().await {
            match data {
                Event::ReadRequest(read) => {
                    read.response
                        .send(Response::Success(vec![56]))
                        .expect("Error sending");
                }
                Event::WriteRequest(_) => todo!(),
                Event::NotifySubscribe(mut notify) => {
                    tokio::spawn(async move {
                        loop {
                            notify
                                .notification
                                .send(vec![56])
                                .await
                                .expect("Error sending");
                            sleep(Duration::from_secs(10)).await
                        }
                    });
                }
                Event::NotifyUnsubscribe => todo!(),
            }
        }
    });

    let battery_level = Characteristic::new(
        Uuid::from_sdp_short_uuid(0x2a19 as u32),
        Properties::new(
            Some(Read(Secure::Insecure(write.clone()))),
            None,
            Some(write),
            None,
        ),
        None,
        HashSet::default(),
    );
    let characteristics = [battery_level].into_iter().collect();

    Service::new(
        Uuid::from_sdp_short_uuid(0x180F as u32),
        true,
        characteristics,
    )
}

fn heart_rate() -> Service {
    let (write, mut read) = channel(1);

    tokio::spawn(async move {
        while let Some(data) = read.next().await {
            if let Event::NotifySubscribe(mut subscribe) = data {
                tokio::spawn(async move {
                    loop {
                        subscribe
                            .notification
                            .send(vec![100])
                            .await
                            .expect("Error sending");
                        sleep(Duration::from_secs(10)).await;
                    }
                });
            }
        }
    });

    let heart_rate_measurement = Characteristic::new(
        Uuid::from_sdp_short_uuid(0x2a37 as u16),
        Properties::new(None, None, Some(write), None),
        None,
        HashSet::default(),
    );

    let characteristics = [heart_rate_measurement].into_iter().collect();

    Service::new(HEART_RATE, true, characteristics)
}

fn bond_service() -> Service {
    let (write, read) = channel(1);

    let bond_management_feature = Characteristic::new(
        uuid_from_sdp(0x2AA5),
        Properties::new(
            Some(Read(Secure::Insecure(write.clone()))),
            None,
            None,
            None,
        ),
        None,
        HashSet::default(),
    );
    let bond_management_control_point = Characteristic::new(
        uuid_from_sdp(0x2AA4),
        Properties::new(
            Some(Read(Secure::Insecure(write.clone()))),
            Some(Write::WithResponse(Secure::Insecure(write))),
            None,
            None,
        ),
        None,
        HashSet::default(),
    );

    let characteristics = [bond_management_feature, bond_management_control_point]
        .into_iter()
        .collect();
    Service::new(BOND, true, characteristics)
}

fn setup_special_service() -> Service {
    let (write, mut read) = channel(1);

    let cmd_to_strap = cmd_to_strap_char();
    let cmd_from_strap = Characteristic::new(
        uuid!("61080003-8d6d-82b8-614a-1c8cb0f8dcc6"),
        Properties::new(None, None, Some(write.clone()), None),
        None,
        HashSet::default(),
    );
    let events_from_strap = Characteristic::new(
        uuid!("61080004-8d6d-82b8-614a-1c8cb0f8dcc6"),
        Properties::new(None, None, Some(write.clone()), None),
        None,
        HashSet::default(),
    );
    let data_from_strap = Characteristic::new(
        uuid!("61080005-8d6d-82b8-614a-1c8cb0f8dcc6"),
        Properties::new(None, None, Some(write.clone()), None),
        None,
        HashSet::default(),
    );
    let memfault = Characteristic::new(
        uuid!("61080007-8d6d-82b8-614a-1c8cb0f8dcc6"),
        Properties::new(None, None, Some(write), None),
        None,
        HashSet::default(),
    );

    let characteristics = [
        cmd_to_strap,
        cmd_from_strap,
        events_from_strap,
        data_from_strap,
        memfault,
    ]
    .into_iter()
    .collect();

    Service::new(SPECIAL_SAUCE, true, characteristics)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // let (sender_characteristic, mut receiver_characteristic) = channel(100);
    // let (sender_descriptor, receiver_descriptor) = channel(1);

    // tokio::spawn(async move {
    //     if let Some(d) = receiver_characteristic.next().await {
    //         match d {
    //             Event::ReadRequest(r) => {
    //                 r.response.send(Response::Success(vec![0xff]));
    //             }
    //             Event::WriteRequest(_) => todo!(),
    //             Event::NotifySubscribe(_) => todo!(),
    //             Event::NotifyUnsubscribe => todo!(),
    //         }
    //     }
    // });

    let peripheral = Peripheral::new().await?;
    peripheral.add_service(&heart_rate())?;
    peripheral.add_service(&battery_service())?;
    peripheral.add_service(&setup_special_service())?;
    peripheral.add_service(&bond_service())?;

    while !peripheral.is_powered().await? {}
    peripheral.register_gatt().await?;
    peripheral
        .start_advertising(ADVERTISING_NAME, &[HEART_RATE])
        .await?;

    while peripheral.is_advertising().await? {}

    Ok(())
}
