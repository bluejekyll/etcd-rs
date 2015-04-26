/// These functional tests are not designed to test the functionality of etcd, but instead that the interaction with
///  it is correct, i.e. that each codepath works. Most likely it would be better to mock most of these tests


use etcd::EtcdClient;
use etcd::etcd_node::EtcdNode;
use etcd::etcd_error::EtcdError;


static TEST_HOST: &'static str = "localhost";
static TEST_PORT: u16 = 4001;

static TEST_DIR: &'static str = "rs_test_dir";
static TEST_KEY: &'static str = "rs_test_dir/rs_test_key";

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

fn client() -> EtcdClient {
    return EtcdClient{etcd_host: TEST_HOST.to_string(), etcd_port: TEST_PORT};
}

/// these are functional tests that need to be executed in order...
#[test]
fn ordered_tests() {
    run!(test_remove_dir(false));
    run!(test_make_dir());
	run!(test_set());
	run!(test_get());
    run!(test_list());
	run!(test_remove());
    run!(test_index_append());

    run!(test_remove_dir(true));
}

fn test_remove_dir(assert_success: bool) {
    let client = client();
    let result = client.remove_dir(TEST_DIR, true);

    if !assert_success {
        return;
    }

    if let Err(e) = result {
        panic!("error: {:?}", e);
    }

    let old_dir = result.unwrap();

    assert!(old_dir.is_some());

    println!("old_dir: {:?}", old_dir);
    assert!(old_dir.unwrap().dir);
}

fn test_make_dir() {
	let client = client();
	let result = client.make_dir(TEST_DIR); // now set it

	if let Err(e) = result {
	    panic!("error: {:?}", e);
    }

    println!("result: {:?}", result);
    assert!(result.unwrap().unwrap().dir)
}

fn test_set() {
	let client = client();
	let result = client.set(TEST_KEY, ""); // null it...
	let result = client.set(TEST_KEY, "testvalue"); // now set it

	if let Err(e) = result {
	  panic!("error: {:?}", e);
    }

    println!("result: {:?}", result);

    assert_eq!(result.unwrap().unwrap().value.unwrap(), ""); // should have been null before because of the first clear...
}

fn test_get() {
	let client = client();

	let result = client.get(TEST_KEY);

	if let Err(e) = result {
	  panic!("error: {:?}", e);
    }

	let object = result.unwrap();

	assert_eq!(object.unwrap().value.unwrap(), "testvalue");
}

fn test_list() {
    let client = client();
    let result = client.get(TEST_DIR);

    if let Err(e) = result {
        panic!("error: {:?}", e);
    }

    let paths = result.unwrap();
    assert!(paths.is_some());

    let paths = paths.unwrap();

    assert!(paths.nodes.is_some());

    let paths = paths.nodes.unwrap();

    assert_eq!(paths.len(), 1);

    // TODO, should really test more than just 1 value...
    let ref zero = paths[0].value;

    assert!(zero.is_some());
    assert_eq!(zero, &Some("testvalue".to_string()));
}

fn test_remove() {
	let client = client();
	let result = client.remove(TEST_KEY); // now set it

	if let Err(e) = result {
    	panic!("error: {:?}", e);
    }

    println!("result: {:?}", result);

    assert_eq!(result.unwrap().unwrap().value.unwrap(), "testvalue");
}

/// this test focuses on creating ordered indexes, it combines both the creation and listing for the
///  tests, which is generally inapropriate.
fn test_index_append() {
    let index: &str = &format!("{}/{}", TEST_DIR, "test_index");
    let client = client();

    let result = client.index_append(index, "test_value1");

    if let Err(e) = result {
        panic!("error: {:?}", e)
    }

    let result = client.index_append(index, "test_value2");

    if let Err(e) = result {
        panic!("error: {:?}", e)
    }

    let result: Result<Option<EtcdNode>, EtcdError> = client.index_list(index);

    if let Err(e) = result {
        panic!("error: {:?}", e)
    }

    let result: Option<EtcdNode> = result.unwrap();
    assert!(result.is_some());

    let result: EtcdNode = result.unwrap();
    assert!(result.nodes.is_some());

    let indexes = result.nodes.unwrap();
    assert_eq!(indexes.len(), 2);

    let ref zero = indexes[0].value;
    assert!(zero.is_some());
    assert_eq!(zero, &Some("test_value1".to_string()));

    let ref one = indexes[1].value;
    assert!(one.is_some());
    assert_eq!(one, &Some("test_value2".to_string()));
}

fn test_rm_dir() {
    let client = EtcdClient{etcd_host: TEST_HOST.to_string(), etcd_port: TEST_PORT};
    let result = client.remove_dir(TEST_DIR, false);

    if let Err(e) = result {
        panic!("error: {:?}", e);
    }

    let value = result.unwrap();
    assert!(value.unwrap().dir);
}
