/// These functional tests are not designed to test the functionality of etcd, but instead that the interaction with
///  it is correct, i.e. that each codepath works. Most likely it would be better to actually mock all of these tests


use etcd::EtcdClient;
use etcd::etcd_node::EtcdNode;

static TEST_HOST: &'static str = "localhost";
static TEST_PORT: u16 = 4001;

/// in order to run the tests in order, but also have an indication of which test we were in when it ran.
macro_rules! run {
  ($function:expr) => (
    {
      println!("testing: {}", stringify!($function));
      $function;
      println!("success: {}", stringify!($function));
    }
  )
}


/// these are functional tests that need to be executed in order...
#[test]
fn ordered_tests() {
	run!(test_set());
	run!(test_get());
	run!(test_remove());
//	run!(test_make());
}

fn test_make_dir() {
	let client = EtcdClient{etcd_host: TEST_HOST.to_string(), etcd_port: TEST_PORT};
	let result = client.make_dir("testdir"); // now set it

	if let Err(e) = result {
	    panic!("error: {:?}", e);
    }

    println!("result: {:?}", result);

    assert!(result.unwrap().unwrap().dir)
}


fn test_remove() {
	let client = EtcdClient{etcd_host: TEST_HOST.to_string(), etcd_port: TEST_PORT};
	let result = client.remove("testkey"); // now set it

	if let Err(e) = result {
    	panic!("error: {:?}", e);
    }

    println!("result: {:?}", result);

    assert_eq!(result.unwrap().unwrap().value.unwrap(), "testvalue");
}

// fn test_create() {
// 	let client = EtcdClient{etcd_host: TEST_HOST.to_string(), etcd_port: TEST_PORT};
// 	let result = client.set("testkey", "testvalue"); // now set it
//
// 	if let Err(e) = result {
// 	  panic!("error: {:?}", e);
//     }
//
//     println!("result: {:?}", result);
//
//     assert_eq!(result.unwrap().unwrap().value.unwrap(), "testvalue");
//
//     let result = client.set("testkey", "testvalue"); // now set it
//
//     if let Err(e) = result {
//       panic!("error: {:?}", e);
//     }
//
//     assert_eq!(result.unwrap().unwrap().value, None);
// }

fn test_set() {
	let client = EtcdClient{etcd_host: TEST_HOST.to_string(), etcd_port: TEST_PORT};
	let result = client.set("testkey", ""); // null it...
	let result = client.set("testkey", "testvalue"); // now set it

	if let Err(e) = result {
	  panic!("error: {:?}", e);
    }

    println!("result: {:?}", result);

    assert_eq!(result.unwrap().unwrap().value.unwrap(), ""); // should have been null before because of the first clear...
}

fn test_get() {
	let client = EtcdClient{etcd_host: TEST_HOST.to_string(), etcd_port: TEST_PORT};

	let result = client.get("testkey");

	if let Err(e) = result {
	  panic!("error: {:?}", e);
    }

	let value = result.unwrap();

	assert_eq!(value.unwrap().value.unwrap(), "testvalue");
}

fn test_rm_dir() {
    let client = EtcdClient{etcd_host: TEST_HOST.to_string(), etcd_port: TEST_PORT};
    let result = client.remove_dir("testdir");

    if let Err(e) = result {
        panic!("error: {:?}", e);
    }

    let value = result.unwrap();
    assert!(value.unwrap().dir);
}
