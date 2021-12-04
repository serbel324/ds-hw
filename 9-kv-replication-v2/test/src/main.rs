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
use std::collections::HashSet;
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
    values: Vec<&'a str>,
    context: Option<&'a str>,
}

#[derive(Serialize)]
struct PutMessage<'a> {
    key: &'a str,
    value: &'a str,
    context: Option<String>,
    quorum: u8,
}

#[derive(Deserialize)]
struct PutRespMessage<'a> {
    key: &'a str,
    values: Vec<&'a str>,
    context: &'a str,
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
    let mut rand = Pcg64::seed_from_u64(config.seed);
    sys.set_delays(0.01, 0.1);
    let mut node_ids = Vec::new();
    for n in 0..config.node_count {
        node_ids.push(format!("{}", n));
    }
    for node_id in node_ids.iter() {
        let node = config.node_factory.build(node_id, (node_id, node_ids.clone()), config.seed);
        sys.add_node(rc!(refcell!(node)));
        let clock_skew = rand.gen_range(0.0 .. 1.0);
        sys.set_clock_skew(node_id, clock_skew);
        // println!("node {} clock skew: {}", node_id, clock_skew);
    }

    return sys;
}

fn check_get(sys: &mut System<JsonMessage>, node: &str, key: &str, quorum: u8,
             expected: Option<Vec<&str>>, max_steps: u32) -> Result<(Vec<String>,Option<String>), String> {
    sys.send_local(JsonMessage::from("GET", &GetMessage {key, quorum}), node);
    let res = sys.step_until_local_message_max_steps(node, max_steps);
    assume!(res.is_ok(), format!("GET_RESP is not returned by {}", node))?;
    let msgs = res.unwrap();
    let msg = msgs.first().unwrap();
    assume_eq!(msg.tip, "GET_RESP")?;
    let data: GetRespMessage = serde_json::from_str(&msg.data).unwrap();
    assume_eq!(data.key, key)?;
    if expected.is_some() {
        let values_set: HashSet<_> = data.values.clone().into_iter().collect();
        let expected_set: HashSet<_> = expected.unwrap().into_iter().collect();
        assume_eq!(values_set, expected_set)?;
    }
    Ok((data.values.iter().map(|x| x.to_string()).collect(), data.context.map(str::to_string)))
}

fn check_put(sys: &mut System<JsonMessage>, node: &str, key: &str, value: &str,
             context: Option<String>, quorum: u8, max_steps: u32) -> Result<(Vec<String>,String), String> {
    sys.send_local(JsonMessage::from("PUT", &PutMessage {key, value, quorum, context }), node);
    let res = sys.step_until_local_message_max_steps(node, max_steps);
    assume!(res.is_ok(), format!("PUT_RESP is not returned by {}", node))?;
    let msgs = res.unwrap();
    let msg = msgs.first().unwrap();
    assume_eq!(msg.tip, "PUT_RESP")?;
    let data: PutRespMessage = serde_json::from_str(&msg.data).unwrap();
    assume_eq!(data.key, key)?;
    Ok((data.values.iter().map(|x| x.to_string()).collect(), data.context.to_string()))
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
    let non_replicas: Vec<String> =
        sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();
    println!("Key {} replicas: {:?}", key, replicas);

    // get key from first node
    check_get(&mut sys, &nodes.get(0).unwrap(), &key, 2, Some(vec![]), 100)?;

    // put key from first replica
    let (values, _) = check_put(&mut sys, &replicas.get(0).unwrap(), &key, &value, None, 2,100)?;
    assume_eq!(values.len(), 1)?;
    assume_eq!(values[0], value)?;

    // get key from last replica
    check_get(&mut sys, &replicas.get(2).unwrap(), &key, 2, Some(vec![&value]), 100)?;

    // get key from first non-replica
    check_get(&mut sys, &non_replicas.get(0).unwrap(), &key, 2, Some(vec![&value]), 100)?;

    // update key from last non-replica
    let (_, ctx) = check_get(
        &mut sys, &non_replicas.get(0).unwrap(), &key, 2, Some(vec![&value]), 100)?;
    let value2 = random_string(8, &mut rand);
    let (values, _) = check_put(&mut sys, &non_replicas.get(2).unwrap(), &key, &value2, ctx, 2, 100)?;
    assume_eq!(values.len(), 1)?;
    assume_eq!(values[0], value2)?;

    // get key from first node
    check_get(&mut sys, &nodes.get(0).unwrap(), &key, 2, Some(vec![&value2]), 100)?;
    Ok(true)
}

fn test_stale_replica(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand).to_uppercase();
    let value = random_string(8, &mut rand);
    let replicas = key_replicas(&key, sys.node_count());
    let non_replicas: Vec<String> =
        sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();

    // put key from non-replica 1 with quorum 3
    check_put(&mut sys, &non_replicas.get(0).unwrap(), &key, &value, None, 3, 100)?;

    // disconnect first replica
    sys.disconnect_node(replicas.get(0).unwrap());

    // update key from last replica with quorum 2
    let (_, ctx) = check_get(&mut sys, replicas.get(2).unwrap(), &key, 2, Some(vec![&value]), 100)?;
    let value2 = random_string(8, &mut rand);
    check_put(&mut sys, &replicas.get(2).unwrap(), &key, &value2, ctx, 2, 100)?;

    // disconnect last replica
    sys.disconnect_node(replicas.get(2).unwrap());
    // connect first replica
    sys.connect_node(replicas.get(0).unwrap());

    // read key from second replica with quorum 2
    check_get(&mut sys, replicas.get(1).unwrap(), &key, 2, Some(vec![&value2]), 100)?;

    // step for a while and check whether first replica got the recent value
    sys.steps(100);
    sys.disconnect_node(replicas.get(0).unwrap());
    check_get(&mut sys, replicas.get(0).unwrap(), &key, 1, Some(vec![&value2]), 100)?;
    Ok(true)
}

fn test_concurrent_writes_1(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand).to_uppercase();
    let replicas = key_replicas(&key, sys.node_count());
    let non_replicas: Vec<String> =
        sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();
    let node1 = &non_replicas.get(0).unwrap();
    let node2 = &non_replicas.get(1).unwrap();
    let node3 = &non_replicas.get(2).unwrap();

    // put key from node1 (quorum=2)
    let value1 = random_string(8, &mut rand);
    let (values, _) = check_put(&mut sys, node1, &key, &value1, None, 2, 100)?;
    assume_eq!(values.len(), 1)?;
    assume_eq!(values[0], value1)?;

    // concurrently (using same context) put key from node2 (quorum=2)
    let value2 = random_string(8, &mut rand);
    let (values, _) = check_put(&mut sys, node2, &key, &value2, None, 2, 100)?;
    assume_eq!(values.len(), 2)?;

    // read key from node3 (quorum=2)
    // should return both values for reconciliation by the client
    check_get(&mut sys, node3, &key, 2, Some(vec![&value1, &value2]), 100)?;
    Ok(true)
}

fn test_concurrent_writes_2(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand).to_uppercase();
    let replicas = key_replicas(&key, sys.node_count());
    let non_replicas: Vec<String> =
        sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();
    let node1 = &non_replicas.get(0).unwrap();
    let node2 = &non_replicas.get(1).unwrap();
    let node3 = &non_replicas.get(2).unwrap();

    // put key from node1 (quorum=2)
    let value1 = random_string(8, &mut rand);
    sys.send_local(JsonMessage::from(
        "PUT", &PutMessage {key: &key, value: &value1, quorum: 2, context: None}), node1);

    // concurrently (using same context) put key from node2 (quorum=2)
    let value2 = random_string(8, &mut rand);
    sys.send_local(JsonMessage::from(
        "PUT", &PutMessage {key: &key, value: &value2, quorum: 2, context: None}), node2);

    // wait until both puts are processed
    let res = sys.step_until_local_message_max_steps(node1, 100);
    assume!(res.is_ok(), format!("PUT_RESP is not returned by {}", node1))?;
    let msgs = res.unwrap();
    let msg = msgs.first().unwrap();
    assume_eq!(msg.tip, "PUT_RESP")?;
    let data: PutRespMessage = serde_json::from_str(&msg.data).unwrap();
    assume_eq!(data.key, key)?;

    let res = sys.step_until_local_message_max_steps(node2, 100);
    assume!(res.is_ok(), format!("PUT_RESP is not returned by {}", node2))?;
    let msgs = res.unwrap();
    let msg = msgs.first().unwrap();
    assume_eq!(msg.tip, "PUT_RESP")?;
    let data: PutRespMessage = serde_json::from_str(&msg.data).unwrap();
    assume_eq!(data.key, key)?;

    // read key from node3 (quorum=2)
    // should return both values for reconciliation by the client
    check_get(&mut sys, node3, &key, 2, Some(vec![&value1, &value2]), 100)?;
    Ok(true)
}

fn test_concurrent_writes_3(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand).to_uppercase();
    let replicas = key_replicas(&key, sys.node_count());
    let non_replicas: Vec<String> =
        sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();

    // put key from replica 1 (quorum=1)
    let value1 = random_string(8, &mut rand);
    check_put(&mut sys, &replicas.get(0).unwrap(), &key, &value1, None, 1, 100)?;

    // concurrently put key from replica 2 (quorum=1)
    let value2 = random_string(8, &mut rand);
    check_put(&mut sys, &replicas.get(1).unwrap(), &key, &value2, None, 1, 100)?;

    // read key from non-replica 1 (quorum=3)
    check_get(&mut sys, &non_replicas.get(0).unwrap(), &key, 3, Some(vec![&value1, &value2]), 100)?;
    Ok(true)
}

fn test_diverged_replicas(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand).to_uppercase();
    let value = random_string(8, &mut rand);
    let replicas = key_replicas(&key, sys.node_count());
    let non_replicas: Vec<String> =
        sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();

    // put key from first replica with quorum 3
    check_put(&mut sys, &replicas.get(0).unwrap(), &key, &value, None, 3, 100)?;

    // disconnect each replica and put value from it
    let mut new_values = Vec::new();
    for replica in replicas.iter() {
        sys.disconnect_node(replica);
        let (_, ctx) = check_get(&mut sys, replica, &key, 1, Some(vec![&value]), 100)?;
        let value2 = random_string(8, &mut rand);
        check_put(&mut sys, replica, &key, &value2, ctx, 1, 100)?;
        new_values.push(value2);
        // read some key to advance time
        // (check that isolated replica is not among this key replicas)
        loop {
            let some_key = random_string(8, &mut rand).to_uppercase();
            if !key_replicas(&some_key, sys.node_count()).contains(&replica) {
                check_get(
                    &mut sys, &non_replicas.get(0).unwrap(), &some_key, 3, Some(vec![]), 100)?;
                break;
            }
        }
        sys.connect_node(replica);
    }

    // read key from first replica with quorum 3
    // should return all three conflicting values
    check_get(
        &mut sys, &replicas.get(0).unwrap(), &key, 3,
        Some(new_values.iter().map(AsRef::as_ref).collect()), 100)?;
    Ok(true)
}

fn test_sloppy_quorum(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = random_string(8, &mut rand).to_uppercase();
    let value = random_string(8, &mut rand);
    let replicas = key_replicas(&key, sys.node_count());
    let non_replicas: Vec<String> =
        sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();

    // put key from first non-replica with quorum 3
    check_put(&mut sys, &non_replicas.get(0).unwrap(), &key, &value, None, 3, 100)?;

    // temporarily disconnect one replica
    sys.disconnect_node(replicas.get(0).unwrap());

    // update key from other non-replica with quorum 3 (should use hinted handoff)
    let (_, ctx) = check_get(
        &mut sys, non_replicas.get(1).unwrap(), &key, 1, Some(vec![&value]), 100)?;
    let value2 = random_string(8, &mut rand);
    check_put(&mut sys, non_replicas.get(1).unwrap(), &key, &value2, ctx, 3, 100)?;

    // read key from other non-replica with quorum 3 (should use hinted handoff)
    check_get(
        &mut sys, non_replicas.get(2).unwrap(), &key, 3, Some(vec![&value2]), 100)?;

    // reconnect first replica and let it receive the update
    sys.connect_node(replicas.get(0).unwrap());
    sys.steps(100);

    // check if first replica got update
    sys.disconnect_node(replicas.get(0).unwrap());
    check_get(
        &mut sys, replicas.get(0).unwrap(), &key, 1, Some(vec![&value2]), 100)?;
    Ok(true)
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
    let non_replicas: Vec<String> =
        sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();
    let non_replica1 = non_replicas.get(0).unwrap();
    let non_replica2 = non_replicas.get(1).unwrap();
    let non_replica3 = non_replicas.get(2).unwrap();

    // put key from some node with quorum 3
    check_put(&mut sys, &nodes.get(0).unwrap(), &key, &value, None, 3, 100)?;

    // partition nodes into two parts
    let part1: Vec<&str> = vec![non_replica1, non_replica2, replica1];
    let part2: Vec<&str> = vec![non_replica3, replica2, replica3];
    sys.make_partition(&part1, &part2);

    // partition 1
    let (values, ctx) = check_get(&mut sys, non_replica1, &key, 2, Some(vec![&value]), 100)?;
    let mut value2 = format!("{}-1", values[0]);
    check_put(&mut sys, non_replica1, &key, &value2, ctx, 2, 100)?;
    let (values, ctx) = check_get(&mut sys, non_replica2, &key, 2, Some(vec![&value2]), 100)?;
    value2 = format!("{}-2", values[0]);
    check_put(&mut sys, non_replica2, &key, &value2, ctx, 2, 100)?;
    check_get(&mut sys, non_replica2, &key, 2, Some(vec![&value2]), 100)?;

    // partition 2
    let (values, ctx) = check_get(&mut sys, non_replica3, &key, 2, Some(vec![&value]), 100)?;
    let value3 = format!("{}-3", values[0]);
    check_put(&mut sys, non_replica3, &key, &value3, ctx, 2, 100)?;
    check_get(&mut sys, non_replica3, &key, 2, Some(vec![&value3]), 100)?;

    // heal partition
    sys.reset_network();
    sys.steps(100);

    // read key from all non-replicas
    check_get(&mut sys, non_replica1, &key, 2, Some(vec![&value2, &value3]), 100)?;
    check_get(&mut sys, non_replica2, &key, 2, Some(vec![&value2, &value3]), 100)?;
    check_get(&mut sys, non_replica3, &key, 2, Some(vec![&value2, &value3]), 100)?;

    // check all replicas
    for replica in replicas.iter() {
        sys.disconnect_node(replica);
        check_get(&mut sys, replica, &key, 1, Some(vec![&value2, &value3]), 100)?;
    }
    Ok(true)
}

fn test_shopping_cart_1(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = format!("cart-{}", random_string(8, &mut rand)).to_uppercase();

    let replicas = key_replicas(&key, sys.node_count());
    let non_replicas: Vec<String> =
        sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();
    let node1 = non_replicas.get(0).unwrap();
    let node2 = non_replicas.get(1).unwrap();

    // node1: + milk
    let mut cart1 = vec!["milk"];
    let (values, ctx1) = check_put(&mut sys, node1, &key, &cart1.join(","), None, 2, 100)?;
    assume!(values.len() == 1)?;
    cart1 = values[0].split(",").collect();

    // node2: + eggs
    let mut cart2 = vec!["eggs"];
    let (values, ctx2) = check_put(&mut sys, node2, &key, &cart2.join(","), None, 2, 100)?;
    assume!(values.len() == 1)?;
    cart2 = values[0].split(",").collect();

    // node1: + flour
    cart1.push("flour");
    let (values, ctx1) = check_put(&mut sys, node1, &key, &cart1.join(","), Some(ctx1), 2, 100)?;
    assume!(values.len() == 1)?;
    cart1 = values[0].split(",").collect();

    // node2: + ham
    cart2.push("ham");
    let (values, _) = check_put(&mut sys, node2, &key, &cart2.join(","), Some(ctx2), 2, 100)?;
    assume!(values.len() == 1)?;

    // node1: + flour
    cart1.push("bacon");
    let (values, _) = check_put(&mut sys, node1, &key, &cart1.join(","), Some(ctx1), 2, 100)?;
    assume!(values.len() == 1)?;

    // read cart from all non-replicas
    let expected: HashSet<_> = vec!["milk", "eggs", "flour", "ham", "bacon"]
        .into_iter().collect();
    for node in non_replicas.iter() {
        let (values, _) = check_get(&mut sys, node, &key, 2, None, 100)?;
        assume!(values.len() == 1)?;
        let values_set: HashSet<_> = values[0].split(",").collect();
        assume_eq!(values_set, expected)?;
    }

    Ok(true)
}

fn test_shopping_cart_2(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = format!("cart-{}", random_string(8, &mut rand)).to_uppercase();

    let replicas = key_replicas(&key, sys.node_count());
    let replica1 = replicas.get(0).unwrap();
    let replica2 = replicas.get(1).unwrap();
    let replica3 = replicas.get(2).unwrap();
    let non_replicas: Vec<String> =
        sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();
    let node1 = non_replicas.get(0).unwrap();
    let node2 = non_replicas.get(1).unwrap();
    let node3 = non_replicas.get(2).unwrap();

    // node1: [beer, snacks]
    let cart0 = vec!["beer", "snacks"];
    let (_, ctx) = check_put(&mut sys, node1, &key, &cart0.join(","), None, 3, 100)?;

    // partition nodes into two parts
    let part1: Vec<&str> = vec![node1, node2, replica1];
    let part2: Vec<&str> = vec![node3, replica2, replica3];
    sys.make_partition(&part1, &part2);

    // partition 1 ---------------------------------------------------------------------------------

    // node1: + milk
    let mut cart1 = cart0.clone();
    cart1.push("milk");
    check_put(&mut sys, node1, &key, &cart1.join(","), Some(ctx), 2, 100)?;
    // node2: read, + eggs
    let (values, ctx) = check_get(&mut sys, node2, &key, 2, Some(vec![&cart1.join(",")]), 100)?;
    let mut cart2: Vec<_> = values[0].split(",").collect();
    cart2.push("eggs");
    check_put(&mut sys, node2, &key, &cart2.join(","), ctx, 2, 100)?;
    // control read
    check_get(&mut sys, node1, &key, 2, Some(vec![&cart2.join(",")]), 100)?;

    // partition 2 ---------------------------------------------------------------------------------

    // node3: read, remove [snacks, beer], + [cheese, wine]
    let (values, ctx) = check_get(&mut sys, node3, &key, 2, Some(vec![&cart0.join(",")]), 100)?;
    let mut cart3: Vec<_> = values[0].split(",").collect();
    cart3.clear();
    cart3.push("cheese");
    cart3.push("wine");
    check_put(&mut sys, node3, &key, &cart3.join(","), ctx, 2, 100)?;
    // control read
    check_get(&mut sys, replica2, &key, 2, Some(vec![&cart3.join(",")]), 100)?;

    // heal partition ------------------------------------------------------------------------------
    sys.reset_network();
    sys.steps(100);

    // read key from all non-replica nodes
    let expected: HashSet<_> = vec!["cheese", "wine", "milk", "eggs", "beer", "snacks"]
        .into_iter().collect();
    for node in non_replicas.iter() {
        let (values, _) = check_get(&mut sys, node, &key, 2, None, 100)?;
        assume!(values.len() == 1)?;
        let values_set: HashSet<_> = values[0].split(",").collect();
        assume_eq!(values_set, expected)?;
    }

    // check all replicas
    for replica in replicas.iter() {
        sys.disconnect_node(replica);
        let (values, _) = check_get(&mut sys, replica, &key, 1, None, 100)?;
        assume!(values.len() == 1)?;
        let values_set: HashSet<_> = values[0].split(",").collect();
        assume_eq!(values_set, expected)?;
    }
    Ok(true)
}

fn test_shopping_xcart_1(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = format!("xcart-{}", random_string(8, &mut rand)).to_uppercase();

    let replicas = key_replicas(&key, sys.node_count());
    let replica1 = replicas.get(0).unwrap();
    let replica2 = replicas.get(1).unwrap();
    let replica3 = replicas.get(2).unwrap();
    let non_replicas: Vec<String> =
        sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();
    let node1 = non_replicas.get(0).unwrap();
    let node2 = non_replicas.get(1).unwrap();
    let node3 = non_replicas.get(2).unwrap();

    // node1: [beer, snacks]
    let cart0 = vec!["beer", "snacks"];
    let (_, ctx) = check_put(&mut sys, node1, &key, &cart0.join(","), None, 3, 100)?;

    // partition nodes into two parts
    let part1: Vec<&str> = vec![node1, node2, replica1];
    let part2: Vec<&str> = vec![node3, replica2, replica3];
    sys.make_partition(&part1, &part2);

    // partition 1 ---------------------------------------------------------------------------------

    // node1: + milk
    let mut cart1 = cart0.clone();
    cart1.push("milk");
    check_put(&mut sys, node1, &key, &cart1.join(","), Some(ctx), 2, 100)?;
    // node2: read, + eggs
    let (values, ctx) = check_get(&mut sys, node2, &key, 2, Some(vec![&cart1.join(",")]), 100)?;
    let mut cart2: Vec<_> = values[0].split(",").collect();
    cart2.push("eggs");
    check_put(&mut sys, node2, &key, &cart2.join(","), ctx, 2, 100)?;
    // control read
    check_get(&mut sys, node1, &key, 2, Some(vec![&cart2.join(",")]), 100)?;

    // partition 2 ---------------------------------------------------------------------------------

    // node3: read, remove [snacks, beer], + [cheese, wine]
    let (values, ctx) = check_get(&mut sys, node3, &key, 2, Some(vec![&cart0.join(",")]), 100)?;
    let mut cart3: Vec<_> = values[0].split(",").collect();
    cart3.clear();
    cart3.push("cheese");
    cart3.push("wine");
    check_put(&mut sys, node3, &key, &cart3.join(","), ctx, 2, 100)?;
    // control read
    check_get(&mut sys, replica2, &key, 2, Some(vec![&cart3.join(",")]), 100)?;

    // heal partition ------------------------------------------------------------------------------
    sys.reset_network();
    sys.steps(100);

    // read key from all non-replica nodes
    let expected: HashSet<_> = vec!["cheese", "wine", "milk", "eggs"]
        .into_iter().collect();
    for node in non_replicas.iter() {
        let (values, _) = check_get(&mut sys, node, &key, 2, None, 100)?;
        assume!(values.len() == 1)?;
        let values_set: HashSet<_> = values[0].split(",").collect();
        assume_eq!(values_set, expected)?;
    }

    // check all replicas
    for replica in replicas.iter() {
        sys.disconnect_node(replica);
        let (values, _) = check_get(&mut sys, replica, &key, 1, None, 100)?;
        assume!(values.len() == 1)?;
        let values_set: HashSet<_> = values[0].split(",").collect();
        assume_eq!(values_set, expected)?;
    }
    Ok(true)
}

fn test_shopping_xcart_2(config: &TestConfig) -> TestResult {
    let mut sys = build_system(config);
    let mut rand = Pcg64::seed_from_u64(config.seed);

    let key = format!("xcart-{}", random_string(8, &mut rand)).to_uppercase();

    let replicas = key_replicas(&key, sys.node_count());
    let replica1 = replicas.get(0).unwrap();
    let replica2 = replicas.get(1).unwrap();
    let replica3 = replicas.get(2).unwrap();
    let non_replicas: Vec<String> =
        sys.get_node_ids().into_iter().filter(|x| !replicas.contains(x)).collect();
    let node1 = non_replicas.get(0).unwrap();
    let node2 = non_replicas.get(1).unwrap();
    let node3 = non_replicas.get(2).unwrap();

    // node1: [lemonade, snacks, beer]
    let cart0 = vec!["lemonade", "snacks", "beer"];
    let (_, ctx) = check_put(&mut sys, node1, &key, &cart0.join(","), None, 3, 100)?;

    // partition nodes into two parts
    let part1: Vec<&str> = vec![node1, node2, replica1];
    let part2: Vec<&str> = vec![node3, replica2, replica3];
    sys.make_partition(&part1, &part2);

    // partition 1 ---------------------------------------------------------------------------------

    // node1: remove lemonade, + milk
    let mut cart1 = cart0.clone();
    cart1.remove(0);
    cart1.push("milk");
    check_put(&mut sys, node1, &key, &cart1.join(","), Some(ctx), 2, 100)?;
    // node2: read, + eggs
    let (values, ctx) = check_get(&mut sys, node2, &key, 2, Some(vec![&cart1.join(",")]), 100)?;
    let mut cart2: Vec<_> = values[0].split(",").collect();
    cart2.push("eggs");
    check_put(&mut sys, node2, &key, &cart2.join(","), ctx, 2, 100)?;
    // control read
    check_get(&mut sys, node1, &key, 2, Some(vec![&cart2.join(",")]), 100)?;

    // partition 2 ---------------------------------------------------------------------------------

    // node3: read, remove [snacks, beer], + [cheese, wine], + snacks (back)
    let (values, ctx) = check_get(&mut sys, node3, &key, 2, Some(vec![&cart0.join(",")]), 100)?;
    let mut cart3: Vec<_> = values[0].split(",").collect();
    cart3.clear();
    cart3.push("lemonade");
    cart3.push("cheese");
    cart3.push("wine");
    let (_, ctx) = check_put(&mut sys, node3, &key, &cart3.join(","), ctx, 2, 100)?;
    cart3.push("snacks");
    check_put(&mut sys, node3, &key, &cart3.join(","), Some(ctx), 2, 100)?;
    // control read
    check_get(&mut sys, replica2, &key, 2, Some(vec![&cart3.join(",")]), 100)?;

    // heal partition ------------------------------------------------------------------------------
    sys.reset_network();
    sys.steps(100);

    // read key from all non-replica nodes
    let expected: HashSet<_> = vec!["milk", "eggs", "wine", "snacks", "cheese"]
        .into_iter().collect();
    for node in non_replicas.iter() {
        let (values, _) = check_get(&mut sys, node, &key, 2, None, 100)?;
        assume!(values.len() == 1)?;
        let values_set: HashSet<_> = values[0].split(",").collect();
        assume_eq!(values_set, expected)?;
    }

    // check all replicas
    for replica in replicas.iter() {
        sys.disconnect_node(replica);
        let (values, _) = check_get(&mut sys, replica, &key, 1, None, 100)?;
        assume!(values.len() == 1)?;
        let values_set: HashSet<_> = values[0].split(",").collect();
        assume_eq!(values_set, expected)?;
    }
    Ok(true)
}

// MAIN --------------------------------------------------------------------------------------------

fn main() {
    let matches = App::new("Replicated KV Store v2 Homework Tests")
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
    tests.add("STALE REPLICA", test_stale_replica, config);
    tests.add("CONCURRENT WRITES 1", test_concurrent_writes_1, config);
    tests.add("CONCURRENT WRITES 2", test_concurrent_writes_2, config);
    tests.add("CONCURRENT WRITES 3", test_concurrent_writes_3, config);
    tests.add("DIVERGED REPLICAS", test_diverged_replicas, config);
    tests.add("SLOPPY QUORUM", test_sloppy_quorum, config);
    tests.add("PARTITIONED CLIENTS", test_partitioned_clients, config);
    tests.add("SHOPPING CART 1", test_shopping_cart_1, config);
    tests.add("SHOPPING CART 2", test_shopping_cart_2, config);
    tests.add("SHOPPING XCART 1", test_shopping_xcart_1, config);
    tests.add("SHOPPING XCART 2", test_shopping_xcart_2, config);

    if test.is_none() {
        tests.run();
    } else {
        tests.run_test(test.unwrap());
    }
}