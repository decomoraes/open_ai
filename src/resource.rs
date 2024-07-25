use std::cell::RefCell;
use std::rc::Rc;
use crate::core::APIClient;

pub type APIResource = Rc<RefCell<APIClient>>;