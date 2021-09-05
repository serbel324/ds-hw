use std::env;
use std::collections::HashSet;
use assertables::{assume, assume_eq};
use clap::{Arg, App, value_t};
use serde::Serialize;
use serde_json::Value;
use sugars::{hset, refcell, rc};

use dslib::system::System;
use dslib::pynode::{JsonMessage, PyNodeFactory};
use dslib::test::{TestSuite, TestResult};

// UTILS -------------------------------------------------------------------------------------------

#[derive(Serialize)]
struct Info<'a> {
    info: &'a str
}

#[derive(Copy, Clone)]
struct TestConfig<'a> {
    sender_f: &'a PyNodeFactory,
    receiver_f: &'a PyNodeFactory,
    seed: u64,
    info_type: &'a str,
    reliable: bool,
    once: bool
}

fn build_system(config: &TestConfig) -> System<JsonMessage> {
    let mut sys = System::with_seed(config.seed);
    let sender = config.sender_f.build("sender", ("sender", "receiver"));
    sys.add_node(rc!(refcell!(sender)));
    let receiver = config.receiver_f.build("receiver", ("receiver",));
    sys.add_node(rc!(refcell!(receiver)));
    return sys;
}

fn check_delivery(sys: &mut System<JsonMessage>, expected: &JsonMessage,
                  reliable: bool, once: bool) -> TestResult {
    let delivered = sys.read_local_messages("receiver");
    assume!(delivered.len() > 0 || !reliable, "Info is not delivered")?;
    if delivered.len() > 0 {
        assume!(delivered.len() == 1 || !once, "Info is delivered more than once")?;
        for msg in delivered {
            assume_eq!(msg.tip, expected.tip, format!("Wrong message type {}", msg.tip))?;
            assume_eq!(msg.data, expected.data, format!("Wrong message content: {}", msg.data))?;
        }
    }
    Ok(true)
}

fn check_order(sys: &mut System<JsonMessage>, expected: &[JsonMessage]) -> TestResult {
    let delivered = sys.read_local_messages("receiver");
    assume_eq!(delivered.len(), expected.len(),
        format!("Wrong count of delivered info: {} (expected {})", delivered.len(), expected.len()))?;
    for i in 0..expected.len() {
        assume_eq!(delivered[i].tip, expected[i].tip, "Wrong message type")?;
        assume_eq!(delivered[i].data, expected[i].data,
            format!("Wrong message content: {} (expected: {})", delivered[i].data, expected[i].data))?;
    }
    Ok(true)
}

// TESTS -------------------------------------------------------------------------------------------

fn test_normal(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let info = JsonMessage::from(config.info_type, &Info { info: "123"});
    sys.send_local(info.clone(), "sender");
    sys.step_until_no_events();
    check_delivery(&mut sys, &info, config.reliable, config.once)
}

fn test_duplicated(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    sys.set_dupl_rate(1.);
    let info = JsonMessage::from(config.info_type, &Info { info: "123"});
    sys.send_local(info.clone(), "sender");
    sys.step_until_no_events();
    check_delivery(&mut sys, &info, config.reliable, config.once)
}

fn test_dropped(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    sys.set_drop_rate(1.);
    let info = JsonMessage::from(config.info_type, &Info { info: "123"});
    sys.send_local(info.clone(), "sender");
    sys.steps(10);
    sys.set_drop_rate(0.);
    sys.step_until_no_events();
    check_delivery(&mut sys, &info, config.reliable, config.once)
}

fn test_order_normal(config: &TestConfig) -> TestResult {
    let count = 5;
    let mut sys = build_system(config);
    let mut infos = Vec::new();
    for i in 0..count {
        let info = JsonMessage::from(config.info_type, &Info { info: &i.to_string()});
        sys.send_local(info.clone(), "sender");
        infos.push(info);
    }
    sys.step_until_no_events();
    check_order(&mut sys, &infos)
}

fn test_order_delayed(config: &TestConfig) -> TestResult {
    let count = 5;
    let mut sys = build_system(config);
    sys.set_delays(1., 5.);
    let mut infos = Vec::new();
    for i in 0..count {
        let info = JsonMessage::from(config.info_type, &Info { info: &i.to_string()});
        sys.send_local(info.clone(), "sender");
        infos.push(info);
    }
    sys.step_until_no_events();
    check_order(&mut sys, &infos)
}

fn test_order_duplicated(config: &TestConfig) -> TestResult {
    let count = 5;
    let mut sys = build_system(config);
    sys.set_dupl_rate(1.);
    let mut infos = Vec::new();
    for i in 0..count {
        let info = JsonMessage::from(config.info_type, &Info { info: &i.to_string()});
        sys.send_local(info.clone(), "sender");
        infos.push(info);
    }
    sys.step_until_no_events();
    check_order(&mut sys, &infos)
}

fn test_order_dropped(config: &TestConfig) -> TestResult {
    let count = 5;
    let mut sys = build_system(config);
    sys.set_drop_rate(1.);
    let mut infos = Vec::new();
    for i in 0..count {
        let info = JsonMessage::from(config.info_type, &Info { info: &i.to_string()});
        sys.send_local(info.clone(), "sender");
        infos.push(info);
    }
    sys.steps(20);
    sys.set_drop_rate(0.);
    sys.step_until_no_events();
    check_order(&mut sys, &infos)
}

// MAIN --------------------------------------------------------------------------------------------

fn main() {
    let matches = App::new("Guarantees Homework Tests")
        .arg(Arg::with_name("solution_path")
           .short("i")
           .long("impl")
           .value_name("PATH")
           .help("Path to Python file with solution")
           .default_value("../solution.py"))
        .arg(Arg::with_name("seed")
           .short("s")
           .long("seed")
           .value_name("SEED")
           .help("Random seed used in tests")
           .default_value("2021"))
        .arg(Arg::with_name("dslib_path")
           .short("l")
           .long("lib")
           .value_name("PATH")
           .help("Path to dslib directory")
           .default_value("../../dslib"))
        .get_matches();
    let solution_path = matches.value_of("solution_path").unwrap();
    let seed = value_t!(matches.value_of("seed"), u64).unwrap();
    let dslib_path = matches.value_of("dslib_path").unwrap();

    env::set_var("PYTHONPATH", format!("{}/python", dslib_path));
    let sender_f = PyNodeFactory::new(solution_path, "Sender");
    let receiver_f = PyNodeFactory::new(solution_path, "Receiver");
    let mut config = TestConfig {
        sender_f: &sender_f,
        receiver_f: &receiver_f,
        seed,
        info_type: "INFO",
        reliable: false,
        once: false
    };
    let mut tests = TestSuite::new();

    // At most once
    config.info_type = "INFO-1";
    config.once = true;
    // without drops should be reliable
    config.reliable = true;
    tests.add("INFO-1 NORMAL", test_normal, config);
    tests.add("INFO-1 DUPLICATED", test_duplicated, config);
    // with drops is not reliable
    config.reliable = false;
    tests.add("INFO-1 DROPPED", test_dropped, config);

    // At least once
    config.info_type = "INFO-2";
    config.reliable = true;
    config.once = false;
    tests.add("INFO-2 NORMAL", test_normal, config);
    tests.add("INFO-2 DUPLICATED", test_duplicated, config);
    tests.add("INFO-2 DROPPED", test_dropped, config);

    // Exactly once
    config.info_type = "INFO-3";
    config.once = true;
    tests.add("INFO-3 NORMAL", test_normal, config);
    tests.add("INFO-3 DUPLICATED", test_duplicated, config);
    tests.add("INFO-3 DROPPED", test_dropped, config);

    // Exactly once + ordered
    config.info_type = "INFO-4";
    tests.add("INFO-4 NORMAL", test_normal, config);
    tests.add("INFO-4 DUPLICATED", test_duplicated, config);
    tests.add("INFO-4 DROPPED", test_dropped, config);
    tests.add("INFO-4 ORDER NORMAL", test_order_normal, config);
    tests.add("INFO-4 ORDER DELAYED", test_order_delayed, config);
    tests.add("INFO-4 ORDER DUPLICATED", test_order_duplicated, config);
    tests.add("INFO-4 ORDER DROPPED", test_order_dropped, config);

    tests.run();
}