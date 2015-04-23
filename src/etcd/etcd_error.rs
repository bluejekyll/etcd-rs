use hyper;
use std::io;
use std::convert::From;
use rustc_serialize::json;

#[derive(Debug)]
pub enum EtcdError {
  Unsuccessful(hyper::status::StatusCode),
  HttpError(hyper::error::HttpError),
  IOError(io::Error),
  DecodingError(json::DecoderError),
  JsonParserError(json::ParserError),
}

impl From<hyper::error::HttpError> for EtcdError {
    fn from(err: hyper::error::HttpError) -> EtcdError {
	   EtcdError::HttpError(err)
    }
}

impl From<io::Error> for EtcdError {
    fn from(err: io::Error) -> EtcdError {
		EtcdError::IOError(err)
	}
}

impl From<json::DecoderError> for EtcdError {
    fn from(err: json::DecoderError) -> EtcdError {
		EtcdError::DecodingError(err)
	}
}

impl From<json::ParserError> for EtcdError {
    fn from(err: json::ParserError) -> EtcdError {
		EtcdError::JsonParserError(err)
	}
}
