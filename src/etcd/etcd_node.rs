use rustc_serialize::json;
//use chrono::datetime::DateTime;
//use chrono::offset::fixed::FixedOffset;

#[derive(Debug)]
pub struct EtcdNode {
  /// key: the HTTP path to which the request was made. etcd uses a file-system-like structure to represent the
  ///   key-value pairs, therefore all keys start with /.
  pub key: String,

  /// createdIndex: an index is a unique, monotonically-incrementing integer created for each change to etcd. This
  ///   specific index reflects the point in the etcd state member at which a given key was created.There are internal
  ///   commands that also change the state behind the scenes, like adding and syncing servers.
  pub created_index: i64,

  /// modifiedIndex: like node.createdIndex, this attribute is also an etcd index. Actions that cause the value to
  ///   change include set, delete, update, create, compareAndSwap and compareAndDelete. Since the get and watch
  ///   commands do not change state in the store, they do not change the value of node.modifiedIndex.
  pub modified_index: i64,

  /// value: the value of the key after resolving the request.
  pub value: Option<String>,

  /// The expiration is the time at which this key will expire and be deleted.
  pub expiration: Option<String>,

  /// The ttl is the specified time to live for the key, in seconds.
  pub ttl: Option<i64>,

  /// this a directory
  pub dir: bool,

  // todo, should this be a map with the key derived from the path of the parent?
  /// the list of nodes in the directory
  pub nodes: Option<Vec<EtcdNode>>,
}

impl EtcdNode {
  pub fn from_json(obj: &json::Object) -> EtcdNode {
	return EtcdNode {
	  key: obj.get("key").unwrap().as_string().unwrap().to_string(),
	  created_index: obj.get("createdIndex").unwrap().as_i64().unwrap(),
	  modified_index: obj.get("modifiedIndex").unwrap().as_i64().unwrap(),
	  value: if let Some(j) = obj.get("value") { Some(j.as_string().unwrap().to_string()) } else { None },
      expiration: if let Some(j) = obj.get("expiration") { Some(j.as_string().unwrap().to_string()) } else { None },
      ttl: if let Some(j) = obj.get("ttl") { j.as_i64() } else { None },
      dir: if let Some(j) = obj.get("dir") { j.as_boolean().unwrap() } else { false },
      nodes: if let Some(j) = obj.get("nodes") {
         let arr: &Vec<json::Json> = j.as_array().unwrap();
         let mut list: Vec<EtcdNode> = Vec::with_capacity(arr.len());

         for n in arr {
           // these should be node
           list.push(EtcdNode::from_json(n.as_object().unwrap()));
          }
         Some(list) // the value
        } else { None },
     }
   }
}

//// "20133-12-04T12:01:21.874888581-08:00"
/*fn decode_tm_from_str<D: Decoder>(d: &mut D) -> Result<DateTime<FixedOffset>, D::Error> {
	let time_str = try!(d.read_str());

	let parse_result = DateTime::parse_from_rfc3339(&time_str);

	println!("parse_result {:?}", parse_result);
	match parse_result {
		Ok(t) => return Ok(t),
		Err(e) => return Err(d.error(e.description())),
	}
}*/


#[cfg(test)]
mod tests {
  use rustc_serialize::json;
  use super::EtcdNode;

  static NODE_JSON: &'static str = "{
                \"createdIndex\": 2,
                \"key\": \"/queue/2\",
                \"modifiedIndex\": 2,
                \"value\": \"Job1\"
            }";

  static COMPLEX_NODE_JSON: &'static str = "{
        \"createdIndex\": 2,
        \"dir\": true,
        \"key\": \"/queue\",
        \"modifiedIndex\": 2,
        \"nodes\": [
            {
                \"createdIndex\": 2,
                \"key\": \"/queue/2\",
                \"modifiedIndex\": 2,
                \"value\": \"Job1\"
            },
            {
                \"createdIndex\": 3,
                \"key\": \"/queue/3\",
                \"modifiedIndex\": 3,
                \"value\": \"Job2\"
            }
        ]
    }";

  #[test]
  fn decode_node_json_test() {
    let json_tree = json::Json::from_str(NODE_JSON).unwrap();
    let etcd_node = EtcdNode::from_json(json_tree.as_object().unwrap());

    assert_eq!(&etcd_node.key as &str, "/queue/2");
    assert_eq!(etcd_node.dir, false);
    assert_eq!(etcd_node.created_index, 2);
    assert_eq!(etcd_node.modified_index, 2);
    assert_eq!(&etcd_node.value.unwrap() as &str, "Job1");
  }

  #[test]
  fn decode_complex_node_json_test() {
    let json_tree = json::Json::from_str(COMPLEX_NODE_JSON).unwrap();
    let etcd_node = EtcdNode::from_json(json_tree.as_object().unwrap());

    assert_eq!(&etcd_node.key as &str, "/queue");
    assert_eq!(etcd_node.dir, true);
    assert_eq!(etcd_node.created_index, 2);
    assert_eq!(etcd_node.modified_index, 2);
    assert_eq!(etcd_node.value, None);

    let ref nodes = etcd_node.nodes.unwrap();

    assert_eq!((&nodes[0]).created_index, 2);
    assert_eq!(&(&nodes[0]).key as &str, "/queue/2");
    assert_eq!((&nodes[0]).modified_index, 2);
    assert_eq!((&nodes[0]).value.as_ref().unwrap() as &str, "Job1");

    assert_eq!((&nodes[1]).created_index, 3);
    assert_eq!((&nodes[1]).key, "/queue/3".to_string());
    assert_eq!((&nodes[1]).modified_index, 3);
    assert_eq!((&nodes[1]).value.as_ref().unwrap()as &str, "Job2");
  }
}
