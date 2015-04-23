use super::etcd_node::EtcdNode;
use rustc_serialize::json;


#[allow(dead_code)]
#[derive(Debug)]
pub struct EtcdResult {
  /// action: the action of the request that was just made.
  pub action: String,

  /// the node upon which the request was made
  pub node: Option<EtcdNode>,

  /// the previous node if there was one
  pub previous_node: Option<EtcdNode>,

  /// X-Etcd-Index is the current etcd index as explained above.
  pub x_etcd_index: i64,

  /// X-Raft-Index is similar to the etcd index but is for the underlying raft protocol
  pub x_raft_index: i64,

  /// X-Raft-Term is an integer that will increase whenever an etcd master election happens in the cluster.
  ///   If this number is increasing rapidly, you may need to tune the election timeout.
  pub x_raft_term: i64,
}

impl EtcdResult {
   pub fn from_json(obj: &json::Object) -> EtcdResult {
	   let node = EtcdResult::node_from_json("node", obj);
	   let prev_node = EtcdResult::node_from_json("prevNode", obj);
	   let result_action: Option<&json::Json> = obj.get("action");

	   return EtcdResult{
	     action: result_action.unwrap().as_string().unwrap().to_string(),
	     node: node,
	     previous_node: prev_node,
		 x_etcd_index: 0, // from headers...
		 x_raft_index: 0,
		 x_raft_term: 0,
	   }
   }

   fn node_from_json(key: &'static str, result_obj: &json::Object) -> Option<EtcdNode> {
	   // get the json for the node
	   let node_obj: &json::Json = match result_obj.get(key) {
           Some(o) => o,
           None => return None,
       };

	   // extract the node
       return match node_obj.as_object() {
		   Some(o) => Some(EtcdNode::from_json(o)),
		   None => None,
		}
   }
}

#[cfg(test)]
mod tests {
  use rustc_serialize::json;
  use super::EtcdResult;
  use etcd::etcd_node::EtcdNode;

  static RESULT_JSON: &'static str = "{
    \"action\": \"expire\",
    \"node\": {
        \"createdIndex\": 8,
        \"key\": \"/dir\",
        \"dir\":true,
        \"modifiedIndex\": 15
    },
    \"prevNode\": {
        \"createdIndex\": 8,
        \"key\": \"/dir\",
        \"dir\":true,
        \"modifiedIndex\": 17,
        \"expiration\": \"2013-12-11T10:39:35.689275857-08:00\"
    }
  }";


  #[test]
  fn decode_result_json_test() {
	let json_tree = json::Json::from_str(RESULT_JSON).unwrap();
	let etcd_result = EtcdResult::from_json(json_tree.as_object().unwrap());

	assert_eq!(etcd_result.action, "expire".to_string());

	let etcd_node = etcd_result.node.as_ref().unwrap();

	assert_eq!(etcd_node.key, "/dir".to_string());
	assert_eq!(etcd_node.dir, true);
	assert_eq!(etcd_node.created_index, 8);
	assert_eq!(etcd_node.modified_index, 15);

	let etcd_prev_node = etcd_result.previous_node.as_ref().unwrap();

	assert_eq!(etcd_prev_node.key, "/dir".to_string());
	assert_eq!(etcd_prev_node.dir, true);
	assert_eq!(etcd_prev_node.created_index, 8);
	assert_eq!(etcd_prev_node.modified_index, 17);
	assert_eq!(*etcd_prev_node.expiration.as_ref().unwrap(), "2013-12-11T10:39:35.689275857-08:00".to_string());
  }

}
