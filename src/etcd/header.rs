static X_ETCD_INDEX: &'static str = "X-Etcd-Index";
static X_RAFT_INDEX: &'static str = "X-Raft-Index";
static X_RAFT_TERM:  &'static str = "X-Raft-Term";

pub struct X_Etcd_Index(u32);
pub struct X_Raft_Index(u32);
pub struct X_Raft_Term(u32);


macro_rules! create_header {

($struct_name:ty, $stat_str:ident) => (

  impl Deref for $struct_name {
    fn deref<'a>(&'a self) -> &'a u32 {
	  let (i) = self;
        return i;
    }
  }

  impl Header for $struct_name {
    fn header_name() -> &' static str {
      return $stat_str;
	}

    fn parse_header(raw: &[Vec<u8>]) -> Option<$struct_name> {
      let num: Option<u32> = from_str(raw);
	  if let Some(i) = num {
	    return Some($struct_name(i));
	  }

	  return None;
    }
  }

  impl HeaderFormat for $struct_name {
    fn fmt_header(& self, fmt: &mut Formatter) -> Result {
	  write!(f, "{}", *self)
	}
  }

  impl Display for $struct_name {
    fn fmt(& self, f: &mut Formatter) -> Result {
	  write!(f, "{}", *self)
	}
  }
)
}