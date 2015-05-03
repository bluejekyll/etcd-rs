mod etcd_error;
mod etcd_node;
mod etcd_result;

#[cfg(test)]
mod tests;

use hyper::client::{Body,Client};
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::header;
use hyper;
use hyper::Url;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::Read;
use etcd::etcd_node::EtcdNode;
use etcd::etcd_result::EtcdResult;
use rustc_serialize::json;
use url;

// etcd protocol version
static VERSION: &'static str = "v2";


/// EtcdObject, i.e. the base Etcd path
enum EtcdObject {
   Version,
   Keys,
   Stats,
}

impl Display for EtcdObject {
  fn fmt(&self, fmtr: &mut Formatter) -> Result<(), fmt::Error> {
	let object_str = match *self {
			EtcdObject::Version => "version",
			EtcdObject::Keys => "keys",
			EtcdObject::Stats => "stats",
		};

	fmtr.write_str(object_str);

	return Ok(())
  }
}

enum AtomicOp<'a> {
   /// The PrevValue must match the specified value.
   PrevValue(&'a str),
   /// The PrevIndex must match the specified index.
   PrevIndex(u64),
   /// True the operation will only succeed if the key already existed, i.e. it's an update, it will fail if it does not
   ///  exist. False if it should not exist, i.e. it's a create, the operation will fail if it already exists.
   PrevExist(bool),
}

impl<'a> AtomicOp<'a> {
   fn into(self) -> (String, String) {
	   match self {
		      AtomicOp::PrevValue(s) => ("prevValue".into(), s.into()),
		      AtomicOp::PrevIndex(i) => ("prevIndex".into(), i.to_string()),
		      AtomicOp::PrevExist(b) => ("prevExists".into(), b.to_string()),
		   }
   }
}

enum Param<'a> {
   Dir(bool),
   Recursive(bool),
   Sorted(bool),
   Value(&'a str),
   Wait(bool),
   WaitIndex(u64),
}

impl<'a> Param<'a> {
    fn into(self) -> (String, String) {
		match self {
				Param::Dir(b) => ("dir".into(), b.to_string()),
				Param::Recursive(b) => ("recursive".into(), b.to_string()),
                Param::Sorted(b) => ("sorted".into(), b.to_string()),
			    Param::Value(s) => ("value".into(), s.into()),
                Param::Wait(b) => ("wait".into(), b.to_string()),
			    Param::WaitIndex(i) => ("waitIndex".into(), i.to_string()),
            }
	}
}



/// EtcdClient for requesting
pub struct EtcdClient {
    etcd_host: String,
    etcd_port: u16,
}

impl EtcdClient {
    fn build_url<'a>(&self, object: EtcdObject, path: &str, params: &'a Vec<(String,String)>) -> hyper::Url {
		// todo this should be https
		let mut url = hyper::Url::parse(&format!("http://{h}:{p}/{v}/{o}/{pt}",
                                                h = self.etcd_host, p = self.etcd_port,
                                                v = VERSION, o = object, pt = path));
        if let Err(e) = url {
            panic!("error parsing url: {}", e);
        }

        let mut url = url.unwrap();
        for i in params.iter().map(|&(ref k,ref v)| -> (&'a str, &'a str) { (k,v) }) {
            println!("query: {:?}", i);
        }
        url.set_query_from_pairs(params.iter().map(|&(ref k,ref v)| -> (&'a str, &'a str) { (k,v) }));
        println!("url: {:?}", url);

        return url;
    }

	#[inline(always)]
	fn accept_json_header() -> hyper::header::Accept {
		return hyper::header::Accept(vec![header::qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))]);
	}

    #[inline(always)]
	fn content_type_form_header() -> hyper::header::ContentType {
	    return hyper::header::ContentType(Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded, vec![]))
    }

	fn to_etcd_result(mut response: hyper::client::response::Response) -> Result<EtcdResult, etcd_error::EtcdError> {
		if !response.status.is_success() {
			// todo should log here.
			return Err(etcd_error::EtcdError::Unsuccessful(response.status));
		}

		let result_object = try!(json::Json::from_reader(&mut response));
		assert!(result_object.is_object(), "expected the result object here");

		let result_object = result_object.as_object().unwrap();
		return Ok(EtcdResult::from_json(result_object));

	}

	// backup		backup an etcd directory
    // cluster-health	check the health of the etcd cluster

    /// make an index value from the specified key (directory) with an ever increasing ordered index.
    fn index_append(&self, key: &str, value: &str) -> Result<Option<EtcdNode>, etcd_error::EtcdError> {
        // TODO this seems like a queue, should we offer queue style operations?
		let url = self.build_url(EtcdObject::Keys, key, &vec![]);
		let body = url::form_urlencoded::serialize_owned(&vec![Param::Value(value).into()]);
		let response: hyper::client::response::Response = try!(Client::new()
															   .post(url)
															   .body(&body as &str)
															   .header(Self::accept_json_header())
                                                               .header(Self::content_type_form_header())
                                                               .send());
		let result = try!(EtcdClient::to_etcd_result(response));

		return Ok(result.node);
	}

    /// this will return the ordered set of indexes on the specified keys
    fn index_list(&self, key: &str) -> Result<Option<EtcdNode>, etcd_error::EtcdError> {
        let url = self.build_url(EtcdObject::Keys, key, &vec![Param::Recursive(true).into(), Param::Sorted(true).into()]);
		let response: hyper::client::response::Response = try!(Client::new()
															   .get(url)
															   .header(EtcdClient::accept_json_header())
															   .send());
		let result = try!(EtcdClient::to_etcd_result(response));

		return Ok(result.node);
	}


    /// make a new directory
	fn make_dir(&self, name: &str) -> Result<Option<EtcdNode>, etcd_error::EtcdError> {
		let url = self.build_url(EtcdObject::Keys, name, &vec![]);
		let body = url::form_urlencoded::serialize_owned(&vec![Param::Dir(true).into()]);
        let response: hyper::client::response::Response = try!(Client::new()
															   .put(url)
															   .body(&body as &str)
                                                               .header(Self::accept_json_header())
															   .header(Self::content_type_form_header())
															   .send());

		let result = try!(EtcdClient::to_etcd_result(response));
		return Ok(result.node);
	}

    /// remove a key
	///  returns the node if it existsed.
	fn remove(&self, key: &str) -> Result<Option<EtcdNode>, etcd_error::EtcdError> {
		let url = self.build_url(EtcdObject::Keys, key, &vec![]);
		let response: hyper::client::response::Response = try!(Client::new()
															   .delete(url)
															   .header(EtcdClient::accept_json_header())
															   .send());
		let result = try!(EtcdClient::to_etcd_result(response));
		return Ok(result.previous_node);
	}

    //// removes the key if it is an empty directory or a key-value pair
	fn remove_dir(&self, dir: &str, recursive: bool) -> Result<Option<EtcdNode>, etcd_error::EtcdError> {
		let url = self.build_url(EtcdObject::Keys, dir, &vec![Param::Dir(true).into(), Param::Recursive(recursive).into()]);
		let response: hyper::client::response::Response = try!(Client::new()
															   .delete(url)
															   .header(EtcdClient::accept_json_header())
															   .send());
		let result = try!(EtcdClient::to_etcd_result(response));
		return Ok(result.previous_node)
	}

    /// retrieve the value of a key
	fn get(&self, key: &str) -> Result<Option<EtcdNode>, etcd_error::EtcdError> {
		println!("getting {}", key);
		let url = self.build_url(EtcdObject::Keys, key, &vec![]);

		let response: hyper::client::response::Response = try!(Client::new()
															   .get(url)
															   .header(EtcdClient::accept_json_header())
															   .send());
		let result = try!(EtcdClient::to_etcd_result(response));

		println!("result {:?}", result);

		return Ok(result.node);
	}

    //// retrieve a directory, this is just a wrapper for get...
	fn list<'a>(&self, dir: &'a str) -> Result<Option<EtcdNode>, etcd_error::EtcdError> {
        return self.get(dir);
    }

    /// set the value of a key
	///  returns the previous node if there was one.
	fn set<'a>(&self, key: &'a str, value: &'a str) -> Result<Option<EtcdNode>, etcd_error::EtcdError> {
		let url = self.build_url(EtcdObject::Keys, key, &vec![Param::Dir(false).into()]);

		let body = url::form_urlencoded::serialize_owned(&vec![Param::Value(value).into()]/*.map(|x| map_str_string(x))*/);
		let response: hyper::client::response::Response = try!(Client::new()
															   .put(url)
															   .body(&body as &str)
															   .header(hyper::header::ContentType(Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded, vec![])))
															   .header(EtcdClient::accept_json_header())
															   .send());
		let result =  try!(EtcdClient::to_etcd_result(response));
		return Ok(result.previous_node);
	}

    //// create a new or existing directory ???
	//fn set_dir(dir: &str) {}

    //// update an existing key with a given value
	//fn update(key: &str, value: &str) {}

    //// update an existing directory
	//fn update_dir(key: &str, value: &Vec<String>) {}

    /// watch a key for changes
    ///  blocks until the key changes
    /// TODO need to accept the wait index option to guarantee no races.
	fn watch<'a>(&self, key: &'a str) -> Result<EtcdResult, etcd_error::EtcdError> {
        let url = self.build_url(EtcdObject::Keys, key, &vec![Param::Wait(true).into()]);

        let response: hyper::client::response::Response = try!(Client::new()
                                                                .get(url)
                                                                .header(EtcdClient::accept_json_header())
                                                                .send());

        // this will block until the server returns, TODO we should really return a future
        return EtcdClient::to_etcd_result(response);
    }


    //// watch a key for changes and exec an executable
	//fn exec_watch(key: String) {}

    // member		member add, remove and list subcommands
    // upgrade		upgrade an old version etcd cluster to a new version
}
