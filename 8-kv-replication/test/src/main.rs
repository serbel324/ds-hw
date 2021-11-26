use std::env;
use assertables::{assume, assume_eq};
use byteorder::{ByteOrder, LittleEndian};
use clap::{Arg, App, value_t};
use env_logger::Builder;
use log::LevelFilter;
use md5;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};
use std::io::Write;
use sugars::{refcell, rc};

use dslib::system::System;
use dslib::pynode::{JsonMessage, PyNodeFactory};
use dslib::test::{TestSuite, TestResult};

// MESSAGES ----------------------------------------------------------------------------------------

#[derive(Serialize)]
struct GetMessage<'a> {
    key: &'a str,
    quorum: u8,
}

#[derive(Deserialize)]
struct GetRespMessage<'a> {
    key: &'a str,
    value: Option<&'a str>,
}

#[derive(Serialize)]
struct PutMessage<'a> {
    key: &'a str,
    value: &'a str,
    quorum: u8,
}

#[derive(Deserialize)]
struct PutRespMessage<'a> {
    key: &'a str,
    value: &'a str,
}

#[derive(Serialize)]
struct DeleteMessage<'a> {
    key: &'a str,
    quorum: u8,
}

#[derive(Deserialize)]
struct DeleteRespMessage<'a> {
    key: &'a str,
    value: Option<&'a str>,
}

// UTILS -------------------------------------------------------------------------------------------

#[derive(Copy, Clone)]
struct TestConfig<'a> {
    node_count: u32,
    node_factory: &'a PyNodeFactory,
    seed: u64,
}

fn init_logger(level: LevelFilter) {
    Builder::new()
        .filter(None, level)
        .format(|buf, record| {
            writeln!(
                buf,
                "{}",
                record.args()
            )
        })
        .init();
}

fn build_system(config: &TestConfig) -> System<JsonMessage> {
    let mut sys = System::with_seed(config.seed);
    sys.set_delays(0.01, 0.1);
    let mut node_ids = Vec::new();
    for n in 0..config.node_count {
        node_ids.push(format!("{}", n));
    }
    for node_id in node_ids.iter() {
        let node = config.node_factory.build(node_id, (node_id, node_ids.clone()), config.seed);
        sys.add_node(rc!(refcell!(node)));
    }
    return sys;
}

fn check_get(sys: &mut System<JsonMessage>, node: &str, key: &str, quorum: u8,
             expected: Option<&str>, max_steps: u32) -> TestResult {
    sys.send_local(JsonMessage::from("GET", &GetMessage {key, quorum}), node);
    let res = sys.step_until_local_message_max_steps(node, max_steps);
    assume!(res.is_ok(), format!("GET_RESP is not returned by {}", node))?;
    let msgs = res.unwrap();
    let msg = msgs.first().unwrap();
    assume_eq!(msg.tip, "GET_RESP")?;
    let data: GetRespMessage = serde_json::from_str(&msg.data).unwrap();
    assume_eq!(data.key, key)?;
    assume_eq!(data.value, expected)?;
    Ok(true)
}

fn check_put(sys: &mut System<JsonMessage>, node: &str, key: &str, value: &str, quorum: u8,
             max_steps: u32) -> TestResult {
    sys.send_local(JsonMessage::from("PUT", &PutMessage {key, value, quorum}), node);
    let res = sys.step_until_local_message_max_steps(node, max_steps);
    assume!(res.is_ok(), format!("PUT_RESP is not returned by {}", node))?;
    let msgs = res.unwrap();
    let msg = msgs.first().unwrap();
    assume_eq!(msg.tip, "PUT_RESP")?;
    let data: PutRespMessage = serde_json::from_str(&msg.data).unwrap();
    assume_eq!(data.key, key)?;
    assume_eq!(data.value, value)?;
    Ok(true)
}

fn check_delete(sys: &mut System<JsonMessage>, node: &str, key: &str, quorum: u8,
             expected: Option<&str>, max_steps: u32) -> TestResult {
    sys.send_local(JsonMessage::from("DELETE", &DeleteMessage {key, quorum}), node);
    let res = sys.step_until_local_message_max_steps(node, max_steps);
    assume!(res.is_ok(), format!("DELETE_RESP is not returned by {}", node))?;
    let msgs = res.unwrap();
    let msg = msgs.first().unwrap();
    assume_eq!(msg.tip, "DELETE_RESP")?;
    let data: DeleteRespMessage = serde_json::from_str(&msg.data).unwrap();
    assume_eq!(data.key, key)?;
    assume_eq!(data.value, expected)?;
    Ok(true)
}

const SYMBOLS: [char; 36] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'
];
const WEIGHTS: [usize; 36] = [
    13, 16, 3, 8, 8, 5, 6, 23, 4, 8, 24, 12, 2, 1, 1, 10, 5, 8,
    10, 1, 24, 3, 1, 8, 12, 22, 5, 20, 18, 5, 5, 2, 1, 3, 16, 22
];

fn random_string(length: usize, rand: &mut Pcg64) -> String {
    let dist = WeightedIndex::new(&WEIGHTS).unwrap();
    rand.sample_iter(&dist).take(length).map(|x| SYMBOLS[x]).collect()
}

fn key_replicas(key: &str, node_count: u32) -> Vec<String> {
    let mut replicas = Vec::new();
    let hash = md5::compute(key);
    let hash128 = LittleEndian::read_u128(&hash.0);
    let mut replica = (hash128 % node_count as u128) as u32;
    for _ in 0..3 {
        replicas.push(replica.to_string());
        replica += 1;
        if replica == node_count {
            replica = 0;
        }
    }
    replicas
}

// TESTS -------------------------------------------------------------------------------------------

fn test_basic(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let nodes = sys.get_node_ids();
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand).to_uppercase();
    let value = random_string(8, &mut rand);
    let replicas = key_replicas(&key, sys.node_count());
    let non_replicas: Vec<String> = sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();
    println!("Key {} replicas: {:?}", key, replicas);

    // get key from first node
    check_get(&mut sys, &nodes.get(0).unwrap(), &key, 2, None, 100)?;

    // put key from first replica
    check_put(&mut sys, &replicas.get(0).unwrap(), &key, &value, 2, 100)?;

    // get key from last replica
    check_get(&mut sys, &replicas.get(2).unwrap(), &key, 2, Some(&value), 100)?;

    // get key from first non-replica
    check_get(&mut sys, &non_replicas.get(0).unwrap(), &key, 2, Some(&value), 100)?;

    // update key from last non-replica
    let value2 = random_string(8, &mut rand);
    check_put(&mut sys, &non_replicas.get(2).unwrap(), &key, &value2, 2, 100)?;

    // get key from first node
    check_get(&mut sys, &nodes.get(0).unwrap(), &key, 2, Some(&value2), 100)?;

    // delete key from second non-replica
    check_delete(&mut sys, &non_replicas.get(0).unwrap(), &key, 2, Some(&value2), 100)?;

    // get key from last replica
    check_get(&mut sys, &replicas.get(2).unwrap(), &key, 2, None, 100)?;

    // get key from first non-replica
    check_get(&mut sys, &non_replicas.get(0).unwrap(), &key, 2, None, 100)
}

fn test_replicas_check(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand);
    let value = random_string(8, &mut rand);
    let replicas = key_replicas(&key, sys.node_count());

    // put key from first replica with quorum 3
    check_put(&mut sys, &replicas.get(0).unwrap(), &key, &value, 3, 100)?;

    // disconnect each replica and check stored value
    for replica in replicas.iter() {
        sys.disconnect_node(replica);
        check_get(&mut sys, replica, &key, 1, Some(&value), 100)?;
    }
    Ok(true)
}

fn test_stale_replica(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand).to_uppercase();
    let value = random_string(8, &mut rand);
    let replicas = key_replicas(&key, sys.node_count());

    // put key from first replica with quorum 3
    check_put(&mut sys, &replicas.get(0).unwrap(), &key, &value, 3, 100)?;

    // disconnect last replica
    sys.disconnect_node(replicas.get(2).unwrap());

    // update key from first replica with quorum 2
    let value2 = random_string(8, &mut rand);
    check_put(&mut sys, &replicas.get(0).unwrap(), &key, &value2, 2, 100)?;

    // disconnect first replica
    sys.disconnect_node(replicas.get(0).unwrap());
    // connect last replica
    sys.connect_node(replicas.get(2).unwrap());

    // read key from second replica with quorum 2
    check_get(&mut sys, replicas.get(1).unwrap(), &key, 2, Some(&value2), 100)?;

    // step for a while and check whether last replica got the recent value
    sys.steps(100);
    sys.disconnect_node(replicas.get(2).unwrap());
    check_get(&mut sys, replicas.get(2).unwrap(), &key, 1, Some(&value2), 100)
}

fn test_diverged_replicas(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand).to_uppercase();
    let value = random_string(8, &mut rand);
    let replicas = key_replicas(&key, sys.node_count());
    let non_replicas: Vec<String> = sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();

    // put key from first replica with quorum 3
    check_put(&mut sys, &replicas.get(0).unwrap(), &key, &value, 3, 100)?;

    // disconnect each replica and put value from it
    let mut new_values = Vec::new();
    for replica in replicas.iter() {
        sys.disconnect_node(replica);
        let value2 = random_string(8, &mut rand);
        check_put(&mut sys, replica, &key, &value2, 1, 100)?;
        new_values.push(value2);
        // make some action to advance time
        // read some key to advance time
        // (check that isolated replica is not among this key replicas)
        loop {
            let some_key = random_string(8, &mut rand).to_uppercase();
            if !key_replicas(&some_key, sys.node_count()).contains(&replica) {
                check_get(&mut sys, &non_replicas.get(0).unwrap(), &some_key, 3, None, 100)?;
                break;
            }
        }
        sys.connect_node(replica);
    }

    // read key from first replica with quorum 3
    let expected = new_values.last().unwrap();
    check_get(&mut sys, &replicas.get(0).unwrap(), &key, 3, Some(&expected), 100)
}

fn test_sloppy_quorum(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let nodes = sys.get_node_ids();
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand).to_uppercase();
    let value = random_string(8, &mut rand);
    let replicas = key_replicas(&key, sys.node_count());

    // put key from first node with quorum 3
    check_put(&mut sys, &nodes.get(0).unwrap(), &key, &value, 3, 100)?;

    // temporarily disconnect one replica
    sys.disconnect_node(replicas.get(0).unwrap());

    // update key from other replica with quorum 3 (should use hinted handoff)
    let value2 = random_string(8, &mut rand);
    check_put(&mut sys, replicas.get(1).unwrap(), &key, &value2, 3, 100)?;

    // read key from other replica with quorum 3 (should use hinted handoff)
    check_get(&mut sys, replicas.get(2).unwrap(), &key, 3, Some(&value2), 100)?;

    // reconnect first replica and let it receive the update
    sys.connect_node(replicas.get(0).unwrap());
    sys.steps(100);

    // check if first replica got update
    sys.disconnect_node(replicas.get(0).unwrap());
    check_get(&mut sys, replicas.get(0).unwrap(), &key, 1, Some(&value2), 100)
}

fn test_partitioned_client(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand).to_uppercase();
    let value = random_string(8, &mut rand);
    let replicas = key_replicas(&key, sys.node_count());
    let non_replicas: Vec<String> = sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();

    // partition clients from all replicas
    let client1 = non_replicas.get(0).unwrap();
    let client2 = non_replicas.get(1).unwrap();
    let g1: Vec<&str> = replicas.iter().map(|s| &**s).collect();
    let g2: Vec<&str> = non_replicas.iter().map(|s| &**s).collect();
    sys.make_partition(&g1, &g2);

    // put key from client1 with quorum 2
    check_put(&mut sys, client1, &key, &value, 2, 100)?;

    // read key from client2 with quorum 2
    check_get(&mut sys, client2, &key, 2, Some(&value), 100)
}

fn test_partitioned_clients(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let nodes = sys.get_node_ids();
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand).to_uppercase();
    let value = random_string(8, &mut rand);
    let replicas = key_replicas(&key, sys.node_count());
    let replica1 = replicas.get(0).unwrap();
    let replica2 = replicas.get(1).unwrap();
    let replica3 = replicas.get(2).unwrap();
    let non_replicas: Vec<String> = sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();
    let client1 = non_replicas.get(0).unwrap();
    let client2 = non_replicas.get(1).unwrap();
    let client3 = non_replicas.get(2).unwrap();

    // put key from first node with quorum 2
    check_put(&mut sys, &nodes.get(0).unwrap(), &key, &value, 2, 100)?;

    // partition clients and replicas
    let part1: Vec<&str> = vec![client1, client2, replica1];
    let part2: Vec<&str> = vec![client3, replica2, replica3];
    sys.make_partition(&part1, &part2);

    // partition 1
    check_get(&mut sys, client1, &key, 2, Some(&value), 100)?;
    let mut value2 = format!("{}-1", value);
    check_put(&mut sys, client1, &key, &value2, 2, 100)?;
    check_get(&mut sys, client2, &key, 2, Some(&value2), 100)?;
    value2 = format!("{}-2", value2);
    check_put(&mut sys, client2, &key, &value2, 2, 100)?;
    check_get(&mut sys, client2, &key, 2, Some(&value2), 100)?;

    // partition 2
    check_get(&mut sys, client3, &key, 2, Some(&value), 100)?;
    let value3 = format!("{}-3", value);
    check_put(&mut sys, client3, &key, &value3, 2, 100)?;
    check_get(&mut sys, client3, &key, 2, Some(&value3), 100)?;

    // heal partition
    sys.reset_network();
    sys.steps(100);

    // read key from all clients
    check_get(&mut sys, client1, &key, 2, Some(&value3), 100)?;
    check_get(&mut sys, client2, &key, 2, Some(&value3), 100)?;
    check_get(&mut sys, client3, &key, 2, Some(&value3), 100)?;

    // check all replicas
    for replica in replicas.iter() {
        sys.disconnect_node(replica);
        check_get(&mut sys, replica, &key, 1, Some(&value3), 100)?;
    }
    Ok(true)
}

// MAIN --------------------------------------------------------------------------------------------

fn main() {
    let matches = App::new("Replicated KV Store Homework Tests")
        .arg(Arg::with_name("solution_path")
           .short("i")
           .long("impl")
           .value_name("PATH")
           .help("Path to Python file with solution")
           .default_value("../solution.py"))
        .arg(Arg::with_name("test")
           .short("t")
           .long("test")
           .value_name("TEST_NAME")
           .help("Test to run (optional)")
           .required(false))
        .arg(Arg::with_name("debug")
           .short("d")
           .long("debug")
           .takes_value(false)
           .help("Print execution trace"))
        .arg(Arg::with_name("node_count")
           .short("n")
           .long("nodes")
           .value_name("N")
           .help("Number of nodes used in tests")
           .default_value("6"))
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
    let node_count = value_t!(matches.value_of("node_count"), u32).unwrap();
    let seed = value_t!(matches.value_of("seed"), u64).unwrap();
    let dslib_path = matches.value_of("dslib_path").unwrap();
    let test = matches.value_of("test");
    if matches.is_present("debug") {
        init_logger(LevelFilter::Trace);
    }

    env::set_var("PYTHONPATH", format!("{}/python", dslib_path));
    env::set_var("PYTHONHASHSEED", seed.to_string());
    let node_factory = PyNodeFactory::new(solution_path, "StorageNode");
    let config = TestConfig {
        node_count,
        node_factory: &node_factory,
        seed
    };
    let mut tests = TestSuite::new();

    tests.add("BASIC", test_basic, config);
    tests.add("REPLICAS CHECK", test_replicas_check, config);
    tests.add("STALE REPLICA", test_stale_replica, config);
    tests.add("DIVERGED REPLICAS", test_diverged_replicas, config);
    tests.add("SLOPPY QUORUM", test_sloppy_quorum, config);
    tests.add("PARTITIONED CLIENT", test_partitioned_client, config);
    tests.add("PARTITIONED CLIENTS", test_partitioned_clients, config);

    if test.is_none() {
        tests.run();
    } else {
        tests.run_test(test.unwrap());
    }
}
